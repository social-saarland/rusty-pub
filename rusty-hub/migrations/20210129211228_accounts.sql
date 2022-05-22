create table accounts (

  id uuid not null default gen_random_uuid(),
  primary_email_id uuid not null,
  created timestamp not null default CURRENT_TIMESTAMP
);

create unique index idx_accounts_id on accounts(id);

create table emails (
  id uuid not null default gen_random_uuid(),
  account_id uuid references accounts (id) on delete cascade, 
  email varchar(500) not null,
  created timestamp not null default CURRENT_TIMESTAMP
);

create unique index idx_emails_id on emails(id);
create unique index idx_emails_email on emails(email);

create type operation as enum (
  '*',
  'view',
  'delete',
  'update'
);



create table groups (
  id uuid not null default gen_random_uuid(),
  name varchar(500) not null
);

create unique index idx_groups_id on groups(id);


create table group_acls (
  account_id uuid references accounts (id) on delete cascade,
  group_id uuid references groups (id) on delete cascade,
  op operation not null
)
