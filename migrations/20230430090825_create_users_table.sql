create table users
(
    user_id   varchar(42) primary key,
    nonce     uuid not null,
    jwt_token text not null
)