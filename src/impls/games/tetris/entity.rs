use crate::impls::games::entities::{Entity, GameEntity};
use crate::ui::Coordinate;

// 每个方块的4个旋转状态（0度、90度、180度、270度）
// 使用 [] 来表示一个方块单元，使其在终端中显示为正方形
pub const TETRIS_PIECES: [[[&str; 4]; 4]; 7] = [
    // I-piece (长条)
    [
        ["        ", "[][][][]", "        ", "        "], // 水平
        ["  []    ", "  []    ", "  []    ", "  []    "], // 垂直
        ["        ", "[][][][]", "        ", "        "], // 水平
        ["  []    ", "  []    ", "  []    ", "  []    "], // 垂直
    ],
    // O-piece (正方形 - 所有旋转状态相同)
    [
        ["[][]", "[][]", "    ", "    "],
        ["[][]", "[][]", "    ", "    "],
        ["[][]", "[][]", "    ", "    "],
        ["[][]", "[][]", "    ", "    "],
    ],
    // T-piece (T形)
    [
        ["  []  ", "[][][]", "      ", "      "], // 0度
        ["  []  ", "  [][]", "  []  ", "      "], // 90度
        ["[][][]", "  []  ", "      ", "      "], // 180度
        ["  []  ", "[][]  ", "  []  ", "      "], // 270度
    ],
    // S-piece (S形)
    [
        ["  [][]", "[][]  ", "      ", "      "], // 0度
        ["[]    ", "[][]  ", "  []  ", "      "], // 90度
        ["  [][]", "[][]  ", "      ", "      "], // 180度
        ["[]    ", "[][]  ", "  []  ", "      "], // 270度
    ],
    // Z-piece (Z形)
    [
        ["[][]  ", "  [][]", "      ", "      "], // 0度
        ["  []  ", "[][]  ", "[]    ", "      "], // 90度
        ["[][]  ", "  [][]", "      ", "      "], // 180度
        ["  []  ", "[][]  ", "[]    ", "      "], // 270度
    ],
    // J-piece (J形)
    [
        ["[]    ", "[][][]", "      ", "      "], // 0度
        ["  [][]", "  []  ", "  []  ", "      "], // 90度
        ["[][][]", "    []", "      ", "      "], // 180度
        ["  []  ", "  []  ", "[][]  ", "      "], // 270度
    ],
    // L-piece (L形)
    [
        ["    []", "[][][]", "      ", "      "], // 0度
        ["  []  ", "  []  ", "  [][]", "      "], // 90度
        ["[][][]", "[]    ", "      ", "      "], // 180度
        ["[][]  ", "  []  ", "  []  ", "      "], // 270度
    ],
];

#[derive(Clone)]
pub struct TetrisPiece {
    pub entity: Entity,
    pub piece_type: usize,
    pub rotation: usize,
    pub color_code: u8,
}

impl TetrisPiece {
    pub fn new(x: u16, y: u16, piece_type: usize) -> Self {
        let display = Self::get_piece_display(piece_type, 0);
        let width = display.lines().map(|line| line.len()).max().unwrap_or(0) as u16;

        Self {
            entity: Entity {
                x,
                y,
                display,
                width,
                last_x: x,
                last_y: y,
            },
            piece_type,
            rotation: 0,
            color_code: (piece_type + 1) as u8,
        }
    }

    fn get_piece_display(piece_type: usize, rotation: usize) -> String {
        let piece_type = piece_type % TETRIS_PIECES.len();
        let rotation = rotation % 4;
        let piece_shapes = &TETRIS_PIECES[piece_type][rotation];
        piece_shapes.join("\n")
    }

    pub fn rotate(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
        let new_display = Self::get_piece_display(self.piece_type, self.rotation);
        self.entity.width = new_display
            .lines()
            .map(|line| line.len())
            .max()
            .unwrap_or(0) as u16;
        self.entity.display = new_display;
    }

    pub fn get_blocks(&self) -> Vec<Coordinate> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = self.entity.display.lines().collect();

        for (row_idx, line) in lines.iter().enumerate() {
            // 每2个字符代表一个方块单元，因为我们使用 "[]" 来表示方块
            let chars: Vec<char> = line.chars().collect();
            let mut col_idx = 0;
            while col_idx < chars.len() {
                // 检查是否是方块的开始 '['
                if col_idx + 1 < chars.len() && chars[col_idx] == '[' && chars[col_idx + 1] == ']' {
                    blocks.push(Coordinate {
                        x: self.entity.x + col_idx as u16,
                        y: self.entity.y + row_idx as u16,
                    });
                    col_idx += 2; // 跳过 "[]" 两个字符
                } else {
                    col_idx += 1;
                }
            }
        }
        blocks
    }
}

impl GameEntity for TetrisPiece {
    fn position(&self) -> Coordinate {
        Coordinate {
            x: self.entity.x,
            y: self.entity.y,
        }
    }

    fn last_position(&self) -> Coordinate {
        Coordinate {
            x: self.entity.last_x,
            y: self.entity.last_y,
        }
    }

    fn move_to(&mut self, x: u16, y: u16) {
        self.entity.last_x = self.entity.x;
        self.entity.last_y = self.entity.y;
        self.entity.x = x;
        self.entity.y = y;
    }

    fn display(&self) -> &str {
        &self.entity.display
    }

    fn width(&self) -> u16 {
        self.entity.width
    }
}

pub struct TetrisBlock {
    pub entity: Entity,
    pub color_code: u8,
}

impl TetrisBlock {
    pub fn new(x: u16, y: u16, color_code: u8) -> Self {
        Self {
            entity: Entity {
                x,
                y,
                display: "[]".to_string(),
                width: 2, // 每个方块占2个字符宽
                last_x: x,
                last_y: y,
            },
            color_code,
        }
    }
}

impl GameEntity for TetrisBlock {
    fn position(&self) -> Coordinate {
        Coordinate {
            x: self.entity.x,
            y: self.entity.y,
        }
    }

    fn last_position(&self) -> Coordinate {
        Coordinate {
            x: self.entity.last_x,
            y: self.entity.last_y,
        }
    }

    fn move_to(&mut self, x: u16, y: u16) {
        self.entity.last_x = self.entity.x;
        self.entity.last_y = self.entity.y;
        self.entity.x = x;
        self.entity.y = y;
    }

    fn display(&self) -> &str {
        &self.entity.display
    }

    fn width(&self) -> u16 {
        self.entity.width
    }
}
