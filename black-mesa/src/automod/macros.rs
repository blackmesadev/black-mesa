#[macro_export]
macro_rules! check_bypass {
    ($self:expr, $config:expr, $ctx:expr, $bypass:expr) => {
        if let Some(bypass) = $bypass {
            if bypass.users.contains(&$ctx.user.id) {
                tracing::debug!("User bypassed automod");
                return Ok(None);
            }

            if bypass.roles.iter().any(|role| $ctx.roles.contains(role)) {
                tracing::debug!("Role bypassed automod");
                return Ok(None);
            }

            let groups = $self.get_user_groups($config, &$ctx.user.id).await;
            groups.iter().for_each(|group| {
                if bypass.groups.contains(&group.name) {
                    tracing::debug!("Group bypassed automod");
                }
            });
        }
    };
}
