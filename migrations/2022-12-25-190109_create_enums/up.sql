CREATE TYPE ProtocolType AS ENUM (
    'GG18'
);

CREATE TYPE KeyType AS ENUM (
    'SignPDF', 'SignChallenge'
);

CREATE TYPE TaskType AS ENUM (
    'Group', 'SignPdf', 'SignChallenge'
);

CREATE TYPE TaskState AS ENUM (
    'Created', 'Running', 'Finished', 'Failed'
);

CREATE TYPE TaskResultType AS ENUM (
    'GroupEstablished', 'Signed'
);