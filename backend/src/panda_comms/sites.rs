// pub struct SiteDetails {
//     pub name: String,
// }

// pub struct Sites {
//     pub sites: Vec<SiteDetails>,
// }

// impl Sites {
//     pub fn build() -> Self {
//         Self { sites: Vec::new() }
//     }

//     pub fn find_site(&self, name: &str) -> Option<&SiteDetails> {
//         self.sites.iter().find(|site| site.name == name)
//     }

//     pub fn register(&mut self, name: String) {
//         if self.find_site(&name).is_none() {
//             self.sites.push(SiteDetails { name });
//         }
//     }

//     pub fn log(&self) {
//         println!("Sites:");
//         for site in &self.sites {
//             println!("  Site: {}", site.name);
//         }
//     }
// }
