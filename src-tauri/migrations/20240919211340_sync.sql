CREATE TABLE sync_branches
(
    id           TEXT                               NOT NULL
        PRIMARY KEY,
    model        TEXT     DEFAULT 'sync_branch'     NOT NULL,
    workspace_id TEXT                               NOT NULL
        REFERENCES workspaces
            ON DELETE CASCADE,
    created_at   DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at   DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    name         TEXT                               NOT NULL,
    commit_ids   TEXT                               NOT NULL
);

CREATE TABLE sync_commits
(
    id           TEXT                               NOT NULL
        PRIMARY KEY,
    model        TEXT     DEFAULT 'sync_commit'     NOT NULL,
    workspace_id TEXT                               NOT NULL
        REFERENCES workspaces
            ON DELETE CASCADE,
    created_at   DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    message      TEXT                               NOT NULL,
    object_ids   TEXT                               NOT NULL
);

CREATE TABLE sync_objects
(
    id           TEXT                               NOT NULL
        PRIMARY KEY,
    model        TEXT     DEFAULT 'sync_object'     NOT NULL,
    workspace_id TEXT                               NOT NULL
        REFERENCES workspaces
            ON DELETE CASCADE,
    created_at   DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    data         BLOB                               NOT NULL,
    model_id     TEXT                               NOT NULL,
    model_model  TEXT                               NOT NULL
);
