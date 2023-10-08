pub enum Path {
    AdminDashboard,
    AdminNewsletter,
    AdminPassword,
    AdminLogout,
    Login,
}

impl TryFrom<&str> for Path {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "admin_dashboard" => Ok(Path::AdminDashboard),
            "admin_newsletter" => Ok(Path::AdminNewsletter),
            "admin_password" => Ok(Path::AdminPassword),
            "admin_logout" => Ok(Path::AdminLogout),
            "login" => Ok(Path::Login),
            _ => Err(anyhow::anyhow!("bad path")),
        }
    }
}

pub fn path_uri(path: Path) -> &'static str {
    match path {
        Path::AdminDashboard => "/admin/dashboard",
        Path::AdminNewsletter => "/admin/newsletters",
        Path::AdminPassword => "/admin/password",
        Path::AdminLogout => "/admin/logout",
        Path::Login => "/login",
    }
}

pub fn get_path(route: &str) -> &'static str {
    let path: Path = route.try_into().expect("bad path");
    path_uri(path)
}
