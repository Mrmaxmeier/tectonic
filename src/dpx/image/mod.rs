#[derive(Debug)]
struct Attributes {
    width, height: isize,
    xdensity, ydensity: f64,
    bbox: PdfRect,

    /* Not appropriate place but... someone need them. */
    page_no: u32,
    page_count: u32,
    bbox_type: u32, // Ugh
    dict: PdfObj,
    tmpfile: char,
}

#[derive(Debug)]
struct XImage {
    ident: String,
    res_name: String, // char[16]
    subtype: ImageSubtype,

    attributes: Attributes,

    filename: String,
    reference: PdfObj,
    resource: PdfObj,
}

#[derive(Debug)]
struct XImageProvider {
    images: HashSet<OsString, XImage>
}
