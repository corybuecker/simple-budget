begin;
alter table accounts add column _id uuid not null default gen_random_uuid();
alter table goals add column _id uuid not null default gen_random_uuid();
alter table envelopes add column _id uuid not null default gen_random_uuid();
alter table users add column _id uuid not null default gen_random_uuid();

alter table accounts drop constraint accounts_user_id_fkey;
alter table envelopes drop constraint envelopes_user_id_fkey;
alter table goals drop constraint goals_user_id_fkey;
alter table sessions drop constraint sessions_user_id_fkey;

alter table users drop constraint users_pkey;
alter table users add primary key (_id);

alter table accounts add column _user_id uuid;
update accounts set _user_id = users._id from users where users.id = accounts.user_id;
alter table accounts alter column _user_id set not null;

alter table envelopes add column _user_id uuid;
update envelopes set _user_id = users._id from users where users.id = envelopes.user_id;
alter table envelopes alter column _user_id set not null;

alter table goals add column _user_id uuid;
update goals set _user_id = users._id from users where users.id = goals.user_id;
alter table goals alter column _user_id set not null;

alter table sessions add column _user_id uuid;
update sessions set _user_id = users._id from users where users.id = sessions.user_id;
alter table sessions alter column _user_id set not null;


alter table accounts add constraint accounts_user_id_fkey FOREIGN KEY (_user_id) REFERENCES users(_id);
alter table accounts drop column user_id;
alter table accounts rename column _user_id to user_id;


alter table envelopes add constraint envelopes_user_id_fkey FOREIGN KEY (_user_id) REFERENCES users(_id);
alter table envelopes drop column user_id;
alter table envelopes rename column _user_id to user_id;

alter table goals add constraint goals_user_id_fkey FOREIGN KEY (_user_id) REFERENCES users(_id);
alter table goals drop column user_id;
alter table goals rename column _user_id to user_id;

alter table sessions add constraint sessions_user_id_fkey FOREIGN KEY (_user_id) REFERENCES users(_id);
alter table sessions drop column user_id;
alter table sessions rename column _user_id to user_id;

alter table accounts drop column id;
alter table accounts rename column _id to id;
alter table accounts add primary key (id);

alter table envelopes drop column id;
alter table envelopes rename column _id to id;
alter table envelopes add primary key (id);

alter table goals drop column id;
alter table goals rename column _id to id;
alter table goals add primary key (id);

alter table users drop column id;
alter table users rename column _id to id;

commit;
