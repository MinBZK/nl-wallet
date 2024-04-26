create database wallet_provider;

create database wallet_server;

create database pid_issuer;

\c wallet_provider
create extension if not exists "uuid-ossp" with schema public;
