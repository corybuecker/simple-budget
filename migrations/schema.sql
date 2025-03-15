--
-- PostgreSQL database dump
--

-- Dumped from database version 17.4 (Debian 17.4-1.pgdg120+2)
-- Dumped by pg_dump version 17.4 (Homebrew)

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
-- Name: Recurrence; Type: TYPE; Schema: public; Owner: simple_budget
--

CREATE TYPE public."Recurrence" AS ENUM (
    'Daily',
    'Weekly',
    'Monthly',
    'Quarterly',
    'Yearly',
    'Never'
);


ALTER TYPE public."Recurrence" OWNER TO simple_budget;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: accounts; Type: TABLE; Schema: public; Owner: simple_budget
--

CREATE TABLE public.accounts (
    id integer NOT NULL,
    user_id integer NOT NULL,
    name text NOT NULL,
    amount numeric NOT NULL,
    debt boolean NOT NULL
);


ALTER TABLE public.accounts OWNER TO simple_budget;

--
-- Name: accounts_id_seq; Type: SEQUENCE; Schema: public; Owner: simple_budget
--

CREATE SEQUENCE public.accounts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.accounts_id_seq OWNER TO simple_budget;

--
-- Name: accounts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: simple_budget
--

ALTER SEQUENCE public.accounts_id_seq OWNED BY public.accounts.id;


--
-- Name: envelopes; Type: TABLE; Schema: public; Owner: simple_budget
--

CREATE TABLE public.envelopes (
    id integer NOT NULL,
    user_id integer NOT NULL,
    name text NOT NULL,
    amount numeric NOT NULL
);


ALTER TABLE public.envelopes OWNER TO simple_budget;

--
-- Name: envelopes_id_seq; Type: SEQUENCE; Schema: public; Owner: simple_budget
--

CREATE SEQUENCE public.envelopes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.envelopes_id_seq OWNER TO simple_budget;

--
-- Name: envelopes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: simple_budget
--

ALTER SEQUENCE public.envelopes_id_seq OWNED BY public.envelopes.id;


--
-- Name: goals; Type: TABLE; Schema: public; Owner: simple_budget
--

CREATE TABLE public.goals (
    id integer NOT NULL,
    user_id integer NOT NULL,
    name text NOT NULL,
    target numeric NOT NULL,
    target_date timestamp with time zone NOT NULL,
    recurrence public."Recurrence" NOT NULL,
    accumulated_amount numeric NOT NULL
);


ALTER TABLE public.goals OWNER TO simple_budget;

--
-- Name: goals_id_seq; Type: SEQUENCE; Schema: public; Owner: simple_budget
--

CREATE SEQUENCE public.goals_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.goals_id_seq OWNER TO simple_budget;

--
-- Name: goals_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: simple_budget
--

ALTER SEQUENCE public.goals_id_seq OWNED BY public.goals.id;


--
-- Name: sessions; Type: TABLE; Schema: public; Owner: simple_budget
--

CREATE TABLE public.sessions (
    id uuid NOT NULL,
    user_id integer NOT NULL,
    expiration timestamp with time zone NOT NULL,
    csrf text NOT NULL
);


ALTER TABLE public.sessions OWNER TO simple_budget;

--
-- Name: users; Type: TABLE; Schema: public; Owner: simple_budget
--

CREATE TABLE public.users (
    id integer NOT NULL,
    subject text NOT NULL,
    email text NOT NULL,
    preferences jsonb
);


ALTER TABLE public.users OWNER TO simple_budget;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: simple_budget
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.users_id_seq OWNER TO simple_budget;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: simple_budget
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: accounts id; Type: DEFAULT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.accounts ALTER COLUMN id SET DEFAULT nextval('public.accounts_id_seq'::regclass);


--
-- Name: envelopes id; Type: DEFAULT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.envelopes ALTER COLUMN id SET DEFAULT nextval('public.envelopes_id_seq'::regclass);


--
-- Name: goals id; Type: DEFAULT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.goals ALTER COLUMN id SET DEFAULT nextval('public.goals_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: accounts accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT accounts_pkey PRIMARY KEY (id);


--
-- Name: envelopes envelopes_pkey; Type: CONSTRAINT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.envelopes
    ADD CONSTRAINT envelopes_pkey PRIMARY KEY (id);


--
-- Name: goals goals_pkey; Type: CONSTRAINT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.goals
    ADD CONSTRAINT goals_pkey PRIMARY KEY (id);


--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_subject_key; Type: CONSTRAINT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_subject_key UNIQUE (subject);


--
-- Name: accounts accounts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT accounts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: envelopes envelopes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.envelopes
    ADD CONSTRAINT envelopes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: goals goals_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.goals
    ADD CONSTRAINT goals_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: sessions sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: simple_budget
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- PostgreSQL database dump complete
--

