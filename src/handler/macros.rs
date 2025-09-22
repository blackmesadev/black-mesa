#[macro_export]
macro_rules! check_permission {
    ($handler:expr, $config:expr, $ctx:expr, $permission:expr) => {
        if !$handler
            .check_permission($config, $ctx, $permission)
            .await?
        {
            if $config.send_permission_denied {
                match $config.prefer_embeds {
                    true => {
                        $handler
                            .send_permission_denied($ctx.channel_id, $permission)
                            .await?
                    }
                    false => {
                        $handler
                            .send_permission_denied_text($ctx.channel_id, $permission)
                            .await?
                    }
                }
            }
            return Ok(());
        }
    };
}

#[macro_export]
macro_rules! check_can_target {
    ($handler:expr, $config:expr, $ctx:expr, $target_ids:expr) => {
        for target_id in $target_ids {
            if !$handler.check_can_target($ctx, &target_id).await? {
                if $config.send_permission_denied {
                    match $config.prefer_embeds {
                        true => $handler.send_cant_target_user($ctx.channel_id).await?,
                        false => $handler.send_cant_target_user_text($ctx.channel_id).await?,
                    }
                }
                return Ok(());
            }
        }
    };
}

#[macro_export]
macro_rules! get_arg {
    ($handler:expr, $config:expr, $ctx:expr, $args:expr, $index:expr, $schema:expr) => {
        match $args.get($index) {
            Some(arg) => arg,
            None => {
                $handler
                    .missing_parameters($config, $ctx, $args, $schema)
                    .await?;
                return Ok(());
            }
        }
    };
}

#[macro_export]
macro_rules! get_raw_arg {
    ($handler:expr, $config:expr, $ctx:expr, $args:expr, $index:expr, $schema:expr) => {
        match $args.get_raw($index) {
            Some(arg) => arg,
            None => {
                $handler
                    .missing_parameters($config, $ctx, $args, $schema)
                    .await?;
                return Ok(());
            }
        }
    };
}
