-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
'ddf8994f-d522-4659-8d02-c1d479057be6',
'harshvseadmin',
'$argon2id$v=19$m=15000,t=2,p=1$QioC3CqZ0+UoVopHZ9mmEQ$RmD8hl0ok+uLSDb8u6ADCLzwN1amaEx7KkO/+QlqVMs'
);
