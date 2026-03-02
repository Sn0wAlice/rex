use anyhow::Result;
use crate::helper::{domain_mail, domain_typosquat};

pub async fn mail_scan(domain: &str) -> Result<()> {
    domain_mail::mail_scan(domain).await
}

pub fn typosquat(domain: &str, output: Option<&str>, method: Option<&str>) -> Result<()> {
    domain_typosquat::generate_list(domain, output, method)
}
