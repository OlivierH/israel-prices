use anyhow::anyhow;
use anyhow::Result;

pub fn to_string(n: &roxmltree::Node) -> String {
    let mut s = match n.text().unwrap_or("") {
        "לא ידוע" | "כללי" | "unknown" | "," => "".to_string(),
        s => s.trim().to_string(),
    };
    s = s.replace('\u{00A0}', " "); // remove non-breaking spaces
    s
}
pub fn to_country_code(n: &roxmltree::Node) -> String {
    let mut s = to_string(n);
    if let Some(country_code) = crate::country_code::to_country_code(&s) {
        s = country_code.to_string();
    }
    s
}
pub fn to_i32(n: &roxmltree::Node) -> Result<i32> {
    Ok(n.text().unwrap_or("0").parse::<i32>()?)
}

pub fn to_child_content(node: &roxmltree::Node, tag: &str) -> Result<String> {
    Ok(to_string(
        &node
            .children()
            .find(|elem| elem.tag_name().name() == tag)
            .ok_or(anyhow!("Couldn't find tag {tag}"))?,
    ))
}

pub fn to_chain_id(node: &roxmltree::Node) -> Result<i64> {
    let chain_id = node.text().unwrap_or("0").parse::<i64>()?;

    Ok({
        if chain_id == 7290058103393 {
            7290696200003
        } else {
            chain_id
        }
    })
}

pub fn get_descendant<'node>(
    node: &'node roxmltree::Document,
    tag: &str,
) -> Option<roxmltree::Node<'node, 'node>> {
    node.descendants().find(|n| n.tag_name().name() == tag)
}
