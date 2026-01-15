/// 棋盘上的单元格状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Black, // 玩家棋子
    White, // AI 棋子
}

/// 棋盘位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub fn new(
        row: usize,
        col: usize,
    ) -> Self {
        Self { row, col }
    }
}

/// 五子棋棋盘
pub struct GomokuBoard {
    size: usize,
    grid: Vec<Vec<Cell>>,
    move_count: usize,
}

impl GomokuBoard {
    /// 创建新棋盘
    pub fn new(size: usize) -> Self {
        let grid = vec![vec![Cell::Empty; size]; size];
        Self {
            size,
            grid,
            move_count: 0,
        }
    }

    /// 获取棋盘大小
    pub fn size(&self) -> usize {
        self.size
    }

    /// 获取指定位置的单元格状态
    pub fn get(
        &self,
        pos: Position,
    ) -> Option<Cell> {
        if pos.row < self.size && pos.col < self.size {
            Some(self.grid[pos.row][pos.col])
        } else {
            None
        }
    }

    /// 在指定位置放置棋子（仅当位置为空时）
    pub fn place(
        &mut self,
        pos: Position,
        cell: Cell,
    ) -> bool {
        if pos.row < self.size && pos.col < self.size && self.grid[pos.row][pos.col] == Cell::Empty {
            self.grid[pos.row][pos.col] = cell;
            self.move_count += 1;
            true
        } else {
            false
        }
    }

    /// 强制在指定位置放置棋子（用于 AI 模拟，不检查位置是否为空）
    pub fn force_place(
        &mut self,
        pos: Position,
        cell: Cell,
    ) {
        if pos.row < self.size && pos.col < self.size {
            // 如果从非空变为空，减少计数
            if self.grid[pos.row][pos.col] != Cell::Empty && cell == Cell::Empty {
                self.move_count -= 1;
            }
            // 如果从空变为非空，增加计数
            else if self.grid[pos.row][pos.col] == Cell::Empty && cell != Cell::Empty {
                self.move_count += 1;
            }
            self.grid[pos.row][pos.col] = cell;
        }
    }

    /// 检查棋盘是否已满
    pub fn is_full(&self) -> bool {
        self.move_count >= self.size * self.size
    }

    /// 获取所有空位
    pub fn empty_positions(&self) -> Vec<Position> {
        let mut positions = Vec::new();
        for row in 0..self.size {
            for col in 0..self.size {
                if self.grid[row][col] == Cell::Empty {
                    positions.push(Position::new(row, col));
                }
            }
        }
        positions
    }

    /// 检查指定位置是否导致获胜（五子连珠）
    pub fn check_win(
        &self,
        pos: Position,
    ) -> bool {
        let cell = match self.get(pos) {
            Some(Cell::Empty) | None => return false,
            Some(c) => c,
        };

        // 检查四个方向：横、竖、主对角线、副对角线
        self.check_direction(pos, cell, 0, 1)   // 横向
            || self.check_direction(pos, cell, 1, 0)   // 竖向
            || self.check_direction(pos, cell, 1, 1)   // 主对角线
            || self.check_direction(pos, cell, 1, -1) // 副对角线
    }

    /// 检查某个方向是否有五子连珠
    fn check_direction(
        &self,
        pos: Position,
        cell: Cell,
        drow: isize,
        dcol: isize,
    ) -> bool {
        let mut count = 1; // 当前位置本身

        // 正方向计数
        count += self.count_consecutive(pos, cell, drow, dcol);
        // 反方向计数
        count += self.count_consecutive(pos, cell, -drow, -dcol);

        count >= 5
    }

    /// 沿指定方向计数连续相同颜色的棋子
    fn count_consecutive(
        &self,
        pos: Position,
        cell: Cell,
        drow: isize,
        dcol: isize,
    ) -> usize {
        let mut count = 0;
        let mut row = pos.row as isize + drow;
        let mut col = pos.col as isize + dcol;

        while row >= 0 && row < self.size as isize && col >= 0 && col < self.size as isize {
            if self.grid[row as usize][col as usize] == cell {
                count += 1;
                row += drow;
                col += dcol;
            } else {
                break;
            }
        }

        count
    }

    /// 评估某个位置的连子数量（用于 AI）
    /// 返回 (己方最大连子数, 对方最大连子数)
    pub fn evaluate_position(
        &self,
        pos: Position,
        player: Cell,
    ) -> (usize, usize) {
        let opponent = match player {
            Cell::Black => Cell::White,
            Cell::White => Cell::Black,
            Cell::Empty => return (0, 0),
        };

        let mut max_player = 0;
        let mut max_opponent = 0;

        // 检查四个方向
        for &(drow, dcol) in &[(0, 1), (1, 0), (1, 1), (1, -1)] {
            let player_count =
                1 + self.count_consecutive(pos, player, drow, dcol) + self.count_consecutive(pos, player, -drow, -dcol);
            let opponent_count = 1
                + self.count_consecutive(pos, opponent, drow, dcol)
                + self.count_consecutive(pos, opponent, -drow, -dcol);

            max_player = max_player.max(player_count);
            max_opponent = max_opponent.max(opponent_count);
        }

        (max_player, max_opponent)
    }

    /// 克隆棋盘（用于 Minimax 搜索）
    pub fn clone_board(&self) -> Self {
        Self {
            size: self.size,
            grid: self.grid.clone(),
            move_count: self.move_count,
        }
    }
}
