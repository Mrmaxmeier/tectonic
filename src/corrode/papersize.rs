/* const */ fn from_mm(length: f64) -> f64 { length * 72.0 / 25.4 }
// const // fn from_in(length: f64) -> f64 { length * 72.0 }

macro_rules! paper_sizes {
    (($w:expr, $h:expr)) => (
        ($w as _, $h as _)
    );
    (($w:expr, $h:expr; mm)) => (
        (from_mm($w as _), from_mm($h as _))
    );
    ( $( $varname:ident: [$i:expr => $size:tt] ),* ) => (
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum PaperSize {
            #[allow(dead_code)]
            Custom(f64, f64),
            $( $varname ),*
        }

        impl PaperSize {
            pub fn dimensions(&self) -> (f64, f64) {
                match *self {
                    PaperSize::Custom(w, h) => (w, h),
                    $( PaperSize::$varname => paper_sizes!($size), )*
                }
            }

            pub fn from_str(ident: &str) -> Option<PaperSize> {
                match ident {
                    $( $i => Some(PaperSize::$varname), )*
                    _ => None,
                }
            }
        }
    );
}

// See: https://github.com/naota/libpaper/blob/master/lib/paperspecs
paper_sizes!(
    A4:            ["a4" => (210, 297; mm)],
    Letter:        ["letter" => (612, 792)],
    Note:          ["note" => (612, 792)],
    Legal:         ["legal" => (612, 1008)],
    Executive:     ["executive" => (522, 756)],
    HalfLetter:    ["halfletter" => (396, 612)],
    HalfExecutive: ["halfexecutive" => (378, 522)],
    Dim11x17:      ["11x17" => (792, 1224)],
    Statement:     ["statement" => (396, 612)],
    Folio:         ["folio" => (612, 936)],
    Quarto:        ["quarto" => (610, 780)],
    Dim10x14:      ["10x14" => (720, 1008)],
    Ledger:        ["ledger" => (1224, 792)],
    Tabloid:       ["tabloid" => (792, 1224)],
    A0:            ["a0" => (841, 1189; mm)],
    A1:            ["a1" => (594, 841; mm)],
    A2:            ["a2" => (420, 594; mm)],
    A3:            ["a3" => (297, 420; mm)],
    A5:            ["a5" => (148, 210; mm)],
    A6:            ["a6" => (105, 148; mm)],
    A7:            ["a7" => (74, 105; mm)],
    A8:            ["a8" => (52, 74; mm)],
    A9:            ["a9" => (37, 52; mm)],
    A10:           ["a10" => (26, 37; mm)],
    B0:            ["b0" => (1000, 1414; mm)],
    B1:            ["b1" => (707, 1000; mm)],
    B2:            ["b2" => (500, 707; mm)],
    B3:            ["b3" => (353, 500; mm)],
    B4:            ["b4" => (250, 353; mm)],
    B5:            ["b5" => (176, 250; mm)],
    B6:            ["b6" => (125, 176; mm)],
    B7:            ["b7" => (88, 125; mm)],
    B8:            ["b8" => (62, 88; mm)],
    B9:            ["b9" => (44, 62; mm)],
    B10:           ["b10" => (31, 44; mm)],
    C2:            ["c2" => (458, 648; mm)],
    C3:            ["c3" => (324, 458; mm)],
    C4:            ["c4" => (229, 354; mm)],
    C5:            ["c5" => (162, 229; mm)],
    C6:            ["c6" => (114, 162; mm)],
    C7:            ["c7" => (81, 114; mm)],
    C8:            ["c8" => (57, 81; mm)],
    DL:            ["DL" => (312, 624)],
    Comm10:        ["Comm10" => (297, 684)],
    Monarch:       ["Monarch" => (279, 540)],
    ArchE:         ["archE" => (2592, 3456)],
    ArchD:         ["archD" => (1728, 2592)],
    ArchC:         ["archC" => (1296, 1728)],
    ArchB:         ["archB" => (864, 1296)],
    ArchA:         ["archA" => (648, 864)],
    Flsa:          ["flsa" => (612, 936)],
    Flse:          ["flse" => (612, 936)],
    CSheet:        ["csheet" => (1224, 1584)],
    DSheet:        ["dsheet" => (1584, 2448)],
    ESheet:        ["esheet" => (2448, 3168)]
);
