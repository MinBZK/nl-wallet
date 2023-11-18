create database wallet_provider;
\c wallet_provider
create extension if not exists "uuid-ossp" with schema public;

create database wallet_server;
