CREATE TABLE msg.message (
    id UUID PRIMARY KEY,
    sender_id UUID NOT NULL REFERENCES platform.user(id) ON DELETE CASCADE,
    receiver_id UUID NOT NULL REFERENCES platform.user(id) ON DELETE CASCADE,
    ADD COLUMN message_type VARCHAR(10) NOT NULL DEFAULT 'text'
        CHECK (message_type IN ('text', 'image')),
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CHECK (sender_id != receiver_id)
);

-- Lookup conversation history. Messages FROM user to friend
CREATE INDEX idx_msg_snd_rcv_ts
    ON msg.message(sender_id, receiver_id, created_at DESC);

-- Lookup conversation history. Messages TO user from friend
CREATE INDEX idx_msg_rcv_snd_ts
    ON msg.message(receiver_id, sender_id, created_at DESC);

-- Lookup most recent message per friend
CREATE INDEX idx_msg_snd_rcv_id_ts
    ON msg.message(sender_id, receiver_id, id, created_at DESC);

CREATE INDEX idx_msg_rcv_snd_id_ts
    ON msg.message(receiver_id, sender_id, id, created_at DESC);