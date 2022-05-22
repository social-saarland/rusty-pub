/*

Provides infrastructure to handle logins via "magic link" emails.

*/

create table login_attempts (
    id uuid not null default gen_random_uuid(),
    email_id uuid not null,
    created timestamp not null default CURRENT_TIMESTAMP
);

create or replace function prepare_login(
    login_email varchar(500)
)
returns uuid as $$
declare
    login_email_id uuid;
    login_id uuid;
begin

    /*  Try to get the id for the email. If no record exists yet,
        create a new one and get the id of the new record.
        */
    login_email_id := (select id from emails where email = login_email);
    if login_email_id is null then
        insert into emails(email) values (login_email) returning id into login_email_id;
    end if;

    /*  Delete possible old existing login attempt(s), create a new entry
        and return the UUID of the record.
        */
    delete from login_attempts where email_id = login_email_id;
    insert into login_attempts(email_id) values (login_email_id) returning id into login_id;

    return login_id;
end;
$$ language plpgsql;


create or replace function execute_login(
    attempt_id uuid
)
returns accounts
as $$
declare
    login_account accounts;
    login_email emails;
begin
    select * from emails where id = (select email_id from login_attempts where id = attempt_id) into login_email;
    if login_email is null then
        return NULL;
    end if;

    if login_email.account_id is NULL then
        insert into accounts (primary_email_id) values (login_email.id) returning * into login_account;
        update emails set account_id = login_account.id where id = login_email.id;
    else
        select * from accounts where id = login_email.account_id into login_account;
    end if;

    return login_account;
end;
$$ language plpgsql;
