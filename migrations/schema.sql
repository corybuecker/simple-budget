--
-- PostgreSQL database dump
--

\restrict pKm1Tt7TSwuNDxIBixP2xaKDCoJoncdfB1mOhrPCuUWYTztIDI22KIyIUy65bXQ

-- Dumped from database version 18.4 (Debian 18.4-1.pgdg13+1)
-- Dumped by pg_dump version 18.4 (Debian 18.4-1.pgdg13+1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: Recurrence; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public."Recurrence" AS ENUM (
    'Daily',
    'Weekly',
    'Monthly',
    'Quarterly',
    'Yearly',
    'Never'
);


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: accounts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.accounts (
    name text NOT NULL,
    amount numeric NOT NULL,
    debt boolean NOT NULL,
    id uuid DEFAULT gen_random_uuid() CONSTRAINT accounts__id_not_null NOT NULL,
    user_id uuid CONSTRAINT accounts__user_id_not_null NOT NULL
);


--
-- Name: envelopes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.envelopes (
    name text NOT NULL,
    amount numeric NOT NULL,
    id uuid DEFAULT gen_random_uuid() CONSTRAINT envelopes__id_not_null NOT NULL,
    user_id uuid CONSTRAINT envelopes__user_id_not_null NOT NULL
);


--
-- Name: goals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.goals (
    name text NOT NULL,
    target numeric NOT NULL,
    target_date timestamp with time zone NOT NULL,
    recurrence public."Recurrence" NOT NULL,
    accumulated_amount numeric NOT NULL,
    start_date timestamp with time zone,
    id uuid DEFAULT gen_random_uuid() CONSTRAINT goals__id_not_null NOT NULL,
    user_id uuid CONSTRAINT goals__user_id_not_null NOT NULL
);


--
-- Name: sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sessions (
    id uuid NOT NULL,
    expiration timestamp with time zone NOT NULL,
    csrf text NOT NULL,
    user_id uuid CONSTRAINT sessions__user_id_not_null NOT NULL
);


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    subject text NOT NULL,
    email text NOT NULL,
    preferences jsonb,
    id uuid DEFAULT gen_random_uuid() CONSTRAINT users__id_not_null NOT NULL
);


--
-- Name: accounts accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT accounts_pkey PRIMARY KEY (id);


--
-- Name: envelopes envelopes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.envelopes
    ADD CONSTRAINT envelopes_pkey PRIMARY KEY (id);


--
-- Name: goals goals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.goals
    ADD CONSTRAINT goals_pkey PRIMARY KEY (id);


--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_subject_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_subject_key UNIQUE (subject);


--
-- Name: accounts accounts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT accounts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: envelopes envelopes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.envelopes
    ADD CONSTRAINT envelopes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: goals goals_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.goals
    ADD CONSTRAINT goals_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: sessions sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- PostgreSQL database dump complete
--

\unrestrict pKm1Tt7TSwuNDxIBixP2xaKDCoJoncdfB1mOhrPCuUWYTztIDI22KIyIUy65bXQ

