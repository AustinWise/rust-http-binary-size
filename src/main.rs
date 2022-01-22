
fn main() -> Result<(), ureq::Error> {
    let body: String = ureq::get("http://example.com")
        .set("Example-Header", "header value")
        .call()?
        .into_string()?;
    println!("got: {}", body);
    Ok(())
}
