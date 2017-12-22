#[derive(Debug)]
pub enum ImageType {
    Jpeg,
    Jp2,
    Png,
    Bmp,
    Pdf,
    Eps,
}

fn source_image_type (meme handle) -> Option<ImageType> {
    let bytes_ = handle.peek_bytes(0x10);
    let bytes: [u8; 0x10] = bytes_;
    /* Original check order: jpeg, jp2, png, bmp, pdf, ps */

    if (bytes[:2] == [0xff, 0xd8]) {
        return Some(ImageType::Jpeg);
    }

    // TODO: handle IMAGE_TYPE_JP2
    // TODO: implement full header checks
    if (bytes[:12] == [
        0x00, 0x00, 0x00, 0x0c,
        0x6a, 0x50, 0x20, 0x20,
        0x0D, 0x0A, 0x87, 0x0A
    ]) {
        return Some(ImageType::Jp2);
    }

    if (bytes[:8] == [137, 80, 78, 71, 13, 10, 26, 10]) {
        return Some(ImageType::Png);
    }

    if (bytes[:2] == ['B' as u8, 'M' as u8]) {
        return Some(ImageType::Bmp);
    }

    if (
        bytes[:8] == "%%PDF-1." &&
        bytes[9] >= '0' as u8 && bytes[9] <= '5' as u8
    ) {
        return Some(ImageType::Pdf);
    }

    if (bytes[:2] == ['%' as u8, '!' as u8]) {
        return Some(ImageType::Eps);
    }

    dpx_warning("Tectonic was unable to detect an image's format");
    return None;
}


