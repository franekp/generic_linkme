pub mod linux {
    use syn::Ident;

    pub fn section(ident: &Ident) -> String {
        format!("generic_linkme_{}", ident)
    }

    pub fn section_start(ident: &Ident) -> String {
        format!("__start_generic_linkme_{}", ident)
    }

    pub fn section_stop(ident: &Ident) -> String {
        format!("__stop_generic_linkme_{}", ident)
    }
}

pub mod freebsd {
    use syn::Ident;

    pub fn section(ident: &Ident) -> String {
        format!("generic_linkme_{}", ident)
    }

    pub fn section_start(ident: &Ident) -> String {
        format!("__start_generic_linkme_{}", ident)
    }

    pub fn section_stop(ident: &Ident) -> String {
        format!("__stop_generic_linkme_{}", ident)
    }
}

pub mod macho {
    use syn::Ident;

    pub fn section(ident: &Ident) -> String {
        format!(
            "__DATA,__glinkm{},regular,no_dead_strip",
            crate::hash(ident),
        )
    }

    pub fn section_start(ident: &Ident) -> String {
        format!("\x01section$start$__DATA$__glinkm{}", crate::hash(ident))
    }

    pub fn section_stop(ident: &Ident) -> String {
        format!("\x01section$end$__DATA$__glinkm{}", crate::hash(ident))
    }
}

pub mod windows {
    use syn::Ident;

    pub fn section(ident: &Ident) -> String {
        format!(".glinkme_{}$b", ident)
    }

    pub fn section_start(ident: &Ident) -> String {
        format!(".glinkme_{}$a", ident)
    }

    pub fn section_stop(ident: &Ident) -> String {
        format!(".glinkme_{}$c", ident)
    }
}

pub mod illumos {
    use syn::Ident;

    pub fn section(ident: &Ident) -> String {
        format!("set_generic_linkme_{}", ident)
    }

    pub fn section_start(ident: &Ident) -> String {
        format!("__start_set_generic_linkme_{}", ident)
    }

    pub fn section_stop(ident: &Ident) -> String {
        format!("__stop_set_generic_linkme_{}", ident)
    }
}
