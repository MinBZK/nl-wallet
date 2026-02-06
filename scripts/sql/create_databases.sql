create database wallet_provider;

create database wallet_provider_audit_log;

create database verification_server;

create database issuance_server;

create database pid_issuer;

\c wallet_provider
create extension if not exists "uuid-ossp" with schema public;

\c wallet_provider_audit_log
create extension if not exists "uuid-ossp" with schema public;

\c verification_server
create extension if not exists "uuid-ossp" with schema public;

\c issuance_server
create extension if not exists "uuid-ossp" with schema public;

\c pid_issuer
create extension if not exists "uuid-ossp" with schema public;
