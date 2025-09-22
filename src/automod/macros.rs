#[macro_export]
macro_rules! check_bypass {
    ($self:expr, $config:expr, $ctx:expr, $bypass:expr) => {
        if let Some(bypass) = $bypass {
            if bypass.users.contains(&$ctx.user.id) {
                return Ok(None);
            }

            if bypass.roles.iter().any(|role| $ctx.roles.contains(role)) {
                return Ok(None);
            }

            let groups = $self.get_user_groups($config, &$ctx.user.id).await;
            for group in &groups {
                if bypass.groups.contains(&group.name) {
                    return Ok(None);
                }
            }
        }
    };
}
