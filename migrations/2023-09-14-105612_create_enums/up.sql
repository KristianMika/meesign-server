CREATE TYPE ProtocolType AS ENUM (
    'Gg18', 'ElGamal', 'Frost'
);

CREATE TYPE KeyType AS ENUM (
    'SignPdf', 'SignChallenge', 'Decrypt'
);

CREATE TYPE TaskType AS ENUM (
    'Group', 'SignPdf', 'SignChallenge', 'Decrypt'
);

CREATE TYPE TaskState AS ENUM (
    'Created', 'Running', 'Finished', 'Failed'
);

CREATE TYPE TaskResultType AS ENUM (
    'GroupEstablished', 'Signed', 'SignedPdf', 'Decrypted'
);