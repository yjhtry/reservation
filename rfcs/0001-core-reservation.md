# Core Reservation Service

- Feature Name: core-reservation-service
- Start Date: 2024-01-07 19:23:10

## Summary

A core reservation services that solves the problem of reserving a resource for a period of time. We leverage postgres EXCLUDE constraints to ensure that only one reservation can be made for a given resource at a given time.

## Motivation

We need a common solution for various reservation requirements: 1) calender booking; 2) hotel/room booing; 3) meeting booking; 4) parking lot booking; 5) etc. Repeatedly building features for these requirements is a waste of time and resources. We need a common solution that can be used for all teams.

## Guide-level explanation

Basic architecture:

![Basic arch](images/arch1.png)

### Service interface

We would use gRPC as a service interface. Below is the proto definition:

```proto
  enum ReservationStatus {
    UNKNOWN = 0;
    PENDING = 1;
    CONFIRMED = 2;
    BLOCKED = 3;
  }

  message Reservation {
    string id = 1;
    string resource_id = 2;
    string user_id = 3;

    ReservationStatus status = 4;
    google.protobuf.Timestamp start = 5;
    google.protobuf.Timestamp end = 6;
    string note = 7
  }

  message ReserveRequest {
    Reservation reservation = 1;
  }

  message ReserveResponse {
    Reservation reservation = 1;
  }

  message updateRequest {
    ReservationStatus status = 1;
    note = 2;
  }

  message UpdateResponse {
    Reservation reservation = 1;
  }

  message ConfirmRequest {
    string id = 1
  }

  message ConfirmResponse {
    Reservation reservation = 1;
  }

  message CancelRequest {
    string id = 1;
  }

  message CancelResponse {
    Reservation reservation = 1;
  }

  message GetRequest {
    string id = 1;
  }

  message GetResponse {
    Reservation reservation = 1;
  }

  message QueryRequest {
    string resource_id = 1;
    string user_id = 2;

    // use status to filter result, If UNKNOWN return all reservations
    ReservationStatus status = 3;
    google.protobuf.Timestamp start = 4;
    google.protobuf.Timestamp end = 5;
  }

  message ListenRequest {}

  message ListenResponse {
    Reservation reservation = 1;
  }

  service ReservationService {
    rpc reserve(ReserveRequest) returns (ReserveResponse);
    rpc confirm(ConfirmRequest) returns (ConfirmResponse);
    rpc update(updateRequest) returns (UpdateResponse);
    rpc cancel(CancelRequest) returns (CancelResponse);
    rpc get(GetRequest) returns (GetResponse);
    rpc query(QueryRequest) returns (stream Reservation);
    // another system can monitor the reservations and newly reserved/confirmed/canceled reservations
    rpc listen(ListenRequest) returns (stream ListenResponse);
  }
```

### Database schema

We would use postgres as the database. Below is the schema:

```sql
CREATE SCHEMA rsvp;
CREATE TYPE rsvp.reservation_status AS ENUM (
  'unknown',
  'pending',
  'confirmed',
  'blocked'
);

CREATE TYPE rsvp.reservation_update_type AS ENUM (
  'unknown',
  'create',
  'update',
  'delete'
);

CREATE OR REPLACE FUNCTION rsvp.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

CREATE TRIGGER update_updated_at BEFORE UPDATE ON rsvp.reservation
FOR EACH ROW EXECUTE PROCEDURE rsvp.update_updated_at_column();

CREATE TABLE rsvp.reservation (
  id UUID NOT NULL DEFAULT uuid_generate_v4(),
  resource_id varchar(64 ) NOT NULL,
  user_id varchar(64) NOT NULL,
  status rsvp.reservation_status NOT NULL DEFAULT 'pending',
  timespan tstzrange NOT NULL,
  note text,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ,

  CONSTRAINT reservation_pkey PRIMARY KEY (id),
  CONSTRAINT reservations_conflict EXCLUDE USING gist (resource_id WITH =, timespan WITH &&)
);

CREATE INDEX reservation_resource_id_idx ON rsvp.reservation (resource_id);

CREATE INDEX reservation_user_id_idx ON rsvp.reservation (user_id);

-- if user_id is null, then return all reservations within the during
-- if resource_id is null, then return all reservations within the during
CREATE OR REPLACE FUNCTION rsvp.query(user_id varchar(64), resource_id varchar(64), during tstzrange) RETURNS TABLE rsvp.reservation AS $$ $$ LANGUAGE plpgsql;

-- reservation change queue
CREATE TABLE rsvp.reservation_change (
  id SERIAL NOT NULL,
  reservation_id varchar(64) NOT NULL,
  op rsvp.reservation_update_type NOT NULL,
);

-- trigger for add/update/delete reservation
CREATE OR REPLACE FUNCTION rsvp.reservation_trigger() RETURNS TRIGGER AS $$
BEGIN
  IF (TG_OP = 'INSERT') THEN
    INSERT INTO rsvp.reservation_change (reservation_id, op) VALUES (NEW.id, 'create');
  ELSIF (TG_OP = 'UPDATE') THEN
    IF (OLD.status <> NEW.status) THEN
      INSERT INTO rsvp.reservation_change (reservation_id, op) VALUES (NEW.id, 'update');
    END IF;
  ELSIF (TG_OP = 'DELETE') THEN
    INSERT INTO rsvp.reservation_change (reservation_id, op) VALUES (OLD.id, 'delete');
  END IF;
  -- notify a channel called reservation_update
  NOTIFY reservation_update;
  RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservation_trigger AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservation
FOR EACH ROW EXECUTE PROCEDURE rsvp.reservation_trigger();
```

## Reference-level explanation

This is the technical portion of the RFC. Explain the design in sufficient detail that:

- Its interaction with other features is clear.
- It is reasonably clear how the feature would be implemented.
- Corner cases are dissected by example.

The section should return to the examples given in the previous section, and explain more fully how the detailed proposal makes those examples work.

## Drawbacks

Why should we *not* do this?

## Rationale and alternatives

- Why is this design the best in the space of possible designs?
- What other designs have been considered and what is the rationale for not choosing them?
- What is the impact of not doing this?
- If this is a language proposal, could this be done in a library or macro instead? Does the proposed change make Rust code easier or harder to read, understand, and maintain?

## Prior art

Discuss prior art, both the good and the bad, in relation to this proposal.
A few examples of what this can include are:

- For language, library, cargo, tools, and compiler proposals: Does this feature exist in other programming languages and what experience have their community had?
- For community proposals: Is this done by some other community and what were their experiences with it?
- For other teams: What lessons can we learn from what other communities have done here?
- Papers: Are there any published papers or great posts that discuss this? If you have some relevant papers to refer to, this can serve as a more detailed theoretical background.

This section is intended to encourage you as an author to think about the lessons from other languages, provide readers of your RFC with a fuller picture.
If there is no prior art, that is fine - your ideas are interesting to us whether they are brand new or if it is an adaptation from other languages.

Note that while precedent set by other languages is some motivation, it does not on its own motivate an RFC.
Please also take into consideration that rust sometimes intentionally diverges from common language features.

## Unresolved questions

- What parts of the design do you expect to resolve through the RFC process before this gets merged?
- What parts of the design do you expect to resolve through the implementation of this feature before stabilization?
- What related issues do you consider out of scope for this RFC that could be addressed in the future independently of the solution that comes out of this RFC?

## Future possibilities

Think about what the natural extension and evolution of your proposal would
be and how it would affect the language and project as a whole in a holistic
way. Try to use this section as a tool to more fully consider all possible
interactions with the project and language in your proposal.
Also consider how this all fits into the roadmap for the project
and of the relevant sub-team.

This is also a good place to "dump ideas", if they are out of scope for the
RFC you are writing but otherwise related.

If you have tried and cannot think of any future possibilities,
you may simply state that you cannot think of anything.

Note that having something written down in the future-possibilities section
is not a reason to accept the current or a future RFC; such notes should be
in the section on motivation or rationale in this or subsequent RFCs.
The section merely provides additional information.
