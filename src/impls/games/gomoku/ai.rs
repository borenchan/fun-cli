use super::entity::{Cell, GomokuBoard, Position};
use rand::Rng;

/// AI 策略 trait
pub trait AIStrategy {
    fn next_move(&self, board: &GomokuBoard) -> Option<Position>;
}

/// 简单 AI：随机选择
pub struct RandomStrategy;

impl AIStrategy for RandomStrategy {
    fn next_move(&self, board: &GomokuBoard) -> Option<Position> {
        let empty_positions = board.empty_positions();
        if empty_positions.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..empty_positions.len());
        Some(empty_positions[index])
    }
}

/// 中等 AI：改进的防守和进攻策略
pub struct DefensiveStrategy;

impl DefensiveStrategy {
    /// 评估在某个位置落子后的得分
    fn evaluate_move(&self, board: &GomokuBoard, pos: Position) -> isize {
        let mut temp_board = board.clone_board();

        // 首先检查 AI 在此位置落子是否获胜
        temp_board.force_place(pos, Cell::White);
        if temp_board.check_win(pos) {
            temp_board.force_place(pos, Cell::Empty);
            return 1000000; // 必胜位置
        }

        // 检查玩家在此位置落子是否获胜（必须阻断）
        temp_board.force_place(pos, Cell::Black);
        if temp_board.check_win(pos) {
            temp_board.force_place(pos, Cell::Empty);
            return 500000; // 必须阻断的位置
        }

        // 评估 AI 在此落子的进攻价值
        temp_board.force_place(pos, Cell::White);
        let ai_score = self.evaluate_threats(&temp_board, pos, Cell::White);

        // 评估玩家在此落子的威胁（需要防守）
        temp_board.force_place(pos, Cell::Black);
        let player_score = self.evaluate_threats(&temp_board, pos, Cell::Black);

        temp_board.force_place(pos, Cell::Empty);

        // 防守稍微比进攻重要
        ai_score + player_score * 2
    }

    /// 评估某个位置的威胁程度
    fn evaluate_threats(&self, board: &GomokuBoard, pos: Position, cell: Cell) -> isize {
        let mut max_threat = 0isize;

        // 检查四个方向
        for &(drow, dcol) in &[(0, 1), (1, 0), (1, 1), (1, -1)] {
            let count = self.count_line(board, pos, cell, drow, dcol);
            let threat = match count {
                5.. => 100000,
                4 => 10000,
                3 => 1000,
                2 => 100,
                _ => 10,
            };
            max_threat = max_threat.max(threat);
        }

        max_threat
    }

    /// 计算某个方向的连子数（包括当前位置）
    fn count_line(
        &self,
        board: &GomokuBoard,
        pos: Position,
        cell: Cell,
        drow: isize,
        dcol: isize,
    ) -> usize {
        let mut count = 1; // 当前位置

        // 正方向
        let mut r = pos.row as isize + drow;
        let mut c = pos.col as isize + dcol;
        while r >= 0
            && r < board.size() as isize
            && c >= 0
            && c < board.size() as isize
            && board
                .get(Position::new(r as usize, c as usize))
                .unwrap()
                == cell
        {
            count += 1;
            r += drow;
            c += dcol;
        }

        // 反方向
        let mut r = pos.row as isize - drow;
        let mut c = pos.col as isize - dcol;
        while r >= 0
            && r < board.size() as isize
            && c >= 0
            && c < board.size() as isize
            && board
                .get(Position::new(r as usize, c as usize))
                .unwrap()
                == cell
        {
            count += 1;
            r -= drow;
            c -= dcol;
        }

        count
    }
}

impl AIStrategy for DefensiveStrategy {
    fn next_move(&self, board: &GomokuBoard) -> Option<Position> {
        let empty_positions = board.empty_positions();
        if empty_positions.is_empty() {
            return None;
        }

        let mut best_pos = empty_positions[0];
        let mut best_score = isize::MIN;

        for &pos in &empty_positions {
            let score = self.evaluate_move(board, pos);
            if score > best_score {
                best_score = score;
                best_pos = pos;
            }
        }

        Some(best_pos)
    }
}

/// 困难 AI：Minimax 搜索
pub struct MinimaxStrategy {
    depth: usize,
    /// 置换表：缓存已评估的局面 (棋盘哈希 -> 评分)
    /// 使用简单的 HashMap，实战中可以用更高效的数据结构
    transposition_table: std::cell::RefCell<std::collections::HashMap<u64, isize>>,
}

/// 棋型评分
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Pattern {
    Five = 100000,      // 五连
    LiveFour = 10000,   // 活四
    DeadFour = 1000,    // 冲四
    LiveThree = 800,    // 活三
    DeadThree = 100,    // 眠三
    LiveTwo = 80,       // 活二
    DeadTwo = 10,       // 眠二
    One = 1,            // 单子
}

impl MinimaxStrategy {
    pub fn new(depth: usize) -> Self {
        Self {
            depth,
            transposition_table: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }

    /// 计算棋盘的简单哈希值（用于置换表）
    fn hash_board(&self, board: &GomokuBoard) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for row in 0..board.size() {
            for col in 0..board.size() {
                let pos = Position::new(row, col);
                let cell = board.get(pos).unwrap();
                // 简单编码：Empty=0, Black=1, White=2
                let code = match cell {
                    Cell::Empty => 0u8,
                    Cell::Black => 1u8,
                    Cell::White => 2u8,
                };
                code.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// 分析某个方向的棋型，返回评分
    fn analyze_pattern(
        &self,
        board: &GomokuBoard,
        pos: Position,
        cell: Cell,
        drow: isize,
        dcol: isize,
    ) -> isize {
        let count = self.count_line(board, pos, cell, drow, dcol);

        if count >= 5 {
            return Pattern::Five as isize;
        }

        if count == 4 {
            // 检查两端是否被堵
            let blocked = self.count_blocks(board, pos, cell, drow, dcol);
            return if blocked == 0 {
                Pattern::LiveFour as isize  // 活四（两端都是空）
            } else {
                Pattern::DeadFour as isize  // 冲四（至少一端被堵）
            };
        }

        if count == 3 {
            let blocked = self.count_blocks(board, pos, cell, drow, dcol);
            return if blocked == 0 {
                Pattern::LiveThree as isize  // 活三
            } else {
                Pattern::DeadThree as isize  // 眠三
            };
        }

        if count == 2 {
            let blocked = self.count_blocks(board, pos, cell, drow, dcol);
            return if blocked == 0 {
                Pattern::LiveTwo as isize
            } else {
                Pattern::DeadTwo as isize
            };
        }

        Pattern::One as isize
    }

    /// 统计某个方向两端被堵的数量（0=两端都空，1=一端被堵，2=两端被堵）
    fn count_blocks(
        &self,
        board: &GomokuBoard,
        pos: Position,
        cell: Cell,
        drow: isize,
        dcol: isize,
    ) -> usize {
        let mut blocks = 0;

        // 检查正方向
        let mut r = pos.row as isize;
        let mut c = pos.col as isize;
        loop {
            r += drow;
            c += dcol;
            if r < 0 || r >= board.size() as isize || c < 0 || c >= board.size() as isize {
                blocks += 1; // 边界算作被堵
                break;
            }
            let next_cell = board.get(Position::new(r as usize, c as usize)).unwrap();
            if next_cell == cell {
                continue;
            } else if next_cell == Cell::Empty {
                break; // 空位，不算被堵
            } else {
                blocks += 1; // 对方棋子，算被堵
                break;
            }
        }

        // 检查反方向
        let mut r = pos.row as isize;
        let mut c = pos.col as isize;
        loop {
            r -= drow;
            c -= dcol;
            if r < 0 || r >= board.size() as isize || c < 0 || c >= board.size() as isize {
                blocks += 1;
                break;
            }
            let next_cell = board.get(Position::new(r as usize, c as usize)).unwrap();
            if next_cell == cell {
                continue;
            } else if next_cell == Cell::Empty {
                break;
            } else {
                blocks += 1;
                break;
            }
        }

        blocks
    }

    /// 计算某个方向的连子数
    fn count_line(
        &self,
        board: &GomokuBoard,
        pos: Position,
        cell: Cell,
        drow: isize,
        dcol: isize,
    ) -> usize {
        let mut count = 1;

        // 正方向
        let mut r = pos.row as isize + drow;
        let mut c = pos.col as isize + dcol;
        while r >= 0
            && r < board.size() as isize
            && c >= 0
            && c < board.size() as isize
            && board
                .get(Position::new(r as usize, c as usize))
                .unwrap()
                == cell
        {
            count += 1;
            r += drow;
            c += dcol;
        }

        // 反方向
        let mut r = pos.row as isize - drow;
        let mut c = pos.col as isize - dcol;
        while r >= 0
            && r < board.size() as isize
            && c >= 0
            && c < board.size() as isize
            && board
                .get(Position::new(r as usize, c as usize))
                .unwrap()
                == cell
        {
            count += 1;
            r -= drow;
            c -= dcol;
        }

        count
    }

    /// 评估在某个位置落子后的得分（使用棋型分析）
    fn evaluate_position(&self, board: &GomokuBoard, pos: Position, cell: Cell) -> isize {
        let mut score = 0isize;
        let mut live_three_count = 0;
        let mut live_four_count = 0;

        // 检查四个方向的棋型
        for &(drow, dcol) in &[(0, 1), (1, 0), (1, 1), (1, -1)] {
            let pattern_score = self.analyze_pattern(board, pos, cell, drow, dcol);
            score += pattern_score;

            // 统计特殊棋型（用于检测双三、四三等）
            if pattern_score == Pattern::LiveThree as isize {
                live_three_count += 1;
            }
            if pattern_score == Pattern::LiveFour as isize {
                live_four_count += 1;
            }
        }

        // 双活三（禁手在标准规则，但这里作为强势棋型）
        if live_three_count >= 2 {
            score += 5000;
        }

        // 四三（一个活四+一个活三）
        if live_four_count >= 1 && live_three_count >= 1 {
            score += 8000;
        }

        score
    }

    /// Minimax 算法（带 Alpha-Beta 剪枝 + 置换表）
    fn minimax(
        &self,
        board: &mut GomokuBoard,
        depth: usize,
        mut alpha: isize,
        mut beta: isize,
        is_maximizing: bool,
        last_move: Option<Position>,
    ) -> isize {
        // 终止条件：检查是否有人获胜
        if let Some(pos) = last_move {
            if board.check_win(pos) {
                return if is_maximizing {
                    -100000 - depth as isize * 100
                } else {
                    100000 + depth as isize * 100
                };
            }
        }

        // 终止条件：达到搜索深度或棋盘已满
        if depth == 0 || board.is_full() {
            return self.evaluate_board(board);
        }

        // 置换表查询：如果这个局面已经评估过，直接返回
        let board_hash = self.hash_board(board);
        if let Some(&cached_score) = self.transposition_table.borrow().get(&board_hash) {
            return cached_score;
        }

        let candidates = self.get_candidate_positions(board);
        if candidates.is_empty() {
            return 0;
        }

        // 移动排序优化：先评估每个位置的静态分数，优先搜索好的走法以提高剪枝效率
        let mut sorted_candidates: Vec<(Position, isize)> = candidates
            .iter()
            .map(|&pos| {
                let cell = if is_maximizing { Cell::White } else { Cell::Black };
                board.force_place(pos, cell);
                let score = self.evaluate_position(board, pos, cell);
                board.force_place(pos, Cell::Empty);
                (pos, score)
            })
            .collect();

        // 最大化时降序，最小化时升序
        if is_maximizing {
            sorted_candidates.sort_by(|a, b| b.1.cmp(&a.1));
        } else {
            sorted_candidates.sort_by(|a, b| a.1.cmp(&b.1));
        }

        let result = if is_maximizing {
            let mut max_eval = isize::MIN;
            for (pos, _) in sorted_candidates {
                board.force_place(pos, Cell::White);
                let eval = self.minimax(board, depth - 1, alpha, beta, false, Some(pos));
                board.force_place(pos, Cell::Empty);

                max_eval = max_eval.max(eval);
                alpha = alpha.max(eval);
                if beta <= alpha {
                    break; // Beta剪枝
                }
            }
            max_eval
        } else {
            let mut min_eval = isize::MAX;
            for (pos, _) in sorted_candidates {
                board.force_place(pos, Cell::Black);
                let eval = self.minimax(board, depth - 1, alpha, beta, true, Some(pos));
                board.force_place(pos, Cell::Empty);

                min_eval = min_eval.min(eval);
                beta = beta.min(eval);
                if beta <= alpha {
                    break; // Alpha剪枝
                }
            }
            min_eval
        };

        // 保存到置换表
        self.transposition_table.borrow_mut().insert(board_hash, result);

        result
    }

    /// 评估整个棋盘
    fn evaluate_board(&self, board: &GomokuBoard) -> isize {
        let mut score = 0isize;

        // 遍历所有棋子，评估局势
        for row in 0..board.size() {
            for col in 0..board.size() {
                let pos = Position::new(row, col);
                match board.get(pos).unwrap() {
                    Cell::White => score += self.evaluate_position(board, pos, Cell::White),
                    Cell::Black => {
                        // 地狱模式：防守权重更高
                        let defense_multiplier = if self.depth >= 6 { 3 } else { 2 };
                        score -= self.evaluate_position(board, pos, Cell::Black) * defense_multiplier;
                    }
                    Cell::Empty => {}
                }
            }
        }

        score
    }

    /// 获取候选位置（智能剪枝 + 威胁优先排序）
    fn get_candidate_positions(&self, board: &GomokuBoard) -> Vec<Position> {
        let empty_positions = board.empty_positions();

        // 如果棋盘几乎空白，只考虑中心区域
        if empty_positions.len() > (board.size() * board.size() * 9 / 10) {
            let center = board.size() / 2;
            return empty_positions
                .into_iter()
                .filter(|&pos| {
                    let row_dist = (pos.row as isize - center as isize).abs();
                    let col_dist = (pos.col as isize - center as isize).abs();
                    row_dist <= 2 && col_dist <= 2
                })
                .collect();
        }

        // 使用 HashSet 去重，性能更好
        let mut candidates = std::collections::HashSet::new();

        for row in 0..board.size() {
            for col in 0..board.size() {
                let pos = Position::new(row, col);
                if board.get(pos).unwrap() != Cell::Empty {
                    // 检查周围2格范围内的空位
                    for dr in -2..=2isize {
                        for dc in -2..=2isize {
                            if dr == 0 && dc == 0 {
                                continue;
                            }
                            let nr = row as isize + dr;
                            let nc = col as isize + dc;
                            if nr >= 0
                                && nr < board.size() as isize
                                && nc >= 0
                                && nc < board.size() as isize
                            {
                                let neighbor = Position::new(nr as usize, nc as usize);
                                if board.get(neighbor).unwrap() == Cell::Empty {
                                    candidates.insert(neighbor);
                                }
                            }
                        }
                    }
                }
            }
        }

        if candidates.is_empty() {
            return empty_positions;
        }

        // 地狱模式：对候选位置按威胁度排序，优先搜索高价值位置
        if self.depth >= 6 {
            let mut scored_candidates: Vec<(Position, isize)> = candidates
                .into_iter()
                .map(|pos| {
                    // 快速评估：只检查直接威胁，不递归
                    let mut temp_board = board.clone_board();
                    temp_board.force_place(pos, Cell::White);
                    let ai_score = if temp_board.check_win(pos) {
                        1000000
                    } else {
                        self.evaluate_position(&temp_board, pos, Cell::White)
                    };

                    temp_board.force_place(pos, Cell::Black);
                    let player_score = if temp_board.check_win(pos) {
                        1000000
                    } else {
                        self.evaluate_position(&temp_board, pos, Cell::Black)
                    };

                    (pos, ai_score.max(player_score))
                })
                .collect();

            // 降序排序，优先搜索高分位置
            scored_candidates.sort_by(|a, b| b.1.cmp(&a.1));

            // 只保留前20个最有价值的位置（大幅减少搜索空间）
            scored_candidates.truncate(20);
            scored_candidates.into_iter().map(|(pos, _)| pos).collect()
        } else {
            candidates.into_iter().collect()
        }
    }
}

impl AIStrategy for MinimaxStrategy {
    fn next_move(&self, board: &GomokuBoard) -> Option<Position> {
        // 清理置换表，避免内存累积
        self.transposition_table.borrow_mut().clear();

        let mut board_clone = board.clone_board();

        // 获取所有空位（不只是候选位置，以确保不会漏掉威胁）
        let all_empty = board.empty_positions();
        if all_empty.is_empty() {
            return None;
        }

        // 首先检查是否有直接获胜的位置（检查所有空位）
        for &pos in &all_empty {
            board_clone.force_place(pos, Cell::White);
            if board_clone.check_win(pos) {
                board_clone.force_place(pos, Cell::Empty);
                return Some(pos);
            }
            board_clone.force_place(pos, Cell::Empty);
        }

        // 检查是否需要阻断对手获胜（检查所有空位，这是关键！）
        for &pos in &all_empty {
            board_clone.force_place(pos, Cell::Black);
            if board_clone.check_win(pos) {
                eprintln!("🚨 AI发现威胁：玩家可在 ({},{}) 获胜，必须阻断！", pos.row + 1, (b'A' + pos.col as u8) as char);
                board_clone.force_place(pos, Cell::Empty);
                return Some(pos); // 必须阻断
            }
            board_clone.force_place(pos, Cell::Empty);
        }

        // 只有在没有紧急威胁时才使用候选位置进行 minimax 搜索
        let candidates = self.get_candidate_positions(board);
        if candidates.is_empty() {
            return all_empty.get(0).copied();
        }

        let mut best_pos = candidates[0];
        let mut best_score = isize::MIN;

        // 使用 minimax 搜索最佳位置
        for &pos in &candidates {
            board_clone.force_place(pos, Cell::White);
            let score = self.minimax(
                &mut board_clone,
                self.depth - 1,
                isize::MIN,
                isize::MAX,
                false,
                Some(pos),
            );
            board_clone.force_place(pos, Cell::Empty);

            if score > best_score {
                best_score = score;
                best_pos = pos;
            }
        }

        Some(best_pos)
    }
}

/// 根据难度选择 AI 策略
pub fn select_strategy(difficulty: u8) -> Box<dyn AIStrategy> {
    match difficulty {
        1 => Box::new(RandomStrategy),
        2 => Box::new(DefensiveStrategy),
        3 => Box::new(MinimaxStrategy::new(4)),  // 困难：深度4
        _ => Box::new(MinimaxStrategy::new(6)),  // 地狱：深度6，搜索更深
    }
}
