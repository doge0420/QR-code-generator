use crate::ec::*;
use crate::encoding::Encoding;
use crate::mask::MaskPattern;
use crate::preprocessor::Preprocessor;

mod bit;
mod debug_utils;
mod ec;
mod encoding;
mod format;
mod mask;
mod preprocessor;
mod qrcode;
mod tables;

fn main() {
    let data = "HELLO WORLD";
    let preprocessor = Preprocessor::new(
        data,
        Encoding::Alphanumeric,
        EcLevel::M,
        MaskPattern::Diagonal,
    );
    let qrcode = preprocessor.generate_qrcode();

    println!("{}", qrcode);
}
