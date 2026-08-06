CREATE TABLE friends.friend_request (
    id UUID PRIMARY KEY,
    sender_id UUID NOT NULL REFERENCES platform.user(id) ON DELETE CASCADE,
    receiver_id UUID NOT NULL REFERENCES platform.user(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'rejected')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Prevent self-friending
    CHECK (sender_id != receiver_id)
);

-- Only one pending request per pair, but
-- rejected pairs can create a new pending request.
CREATE UNIQUE INDEX idx_friend_request_pending
    ON business.friend_request(sender_id, receiver_id)
    WHERE status = 'pending';

-- Lookups for pending received requests
CREATE INDEX idx_friend_request_receiver_status
    ON business.friend_request(receiver_id, status)
    WHERE status = 'pending';

-- Lookups for pending sent requests
CREATE INDEX idx_friend_request_sender_status
    ON business.friend_request(sender_id, status)
    WHERE status = 'pending';

-- Lookups for accepted friendships for the sender
CREATE INDEX idx_friend_request_accepted_sender
    ON business.friend_request(sender_id, status)
    WHERE status = 'accepted';

-- Lookups for accepted friendships for the receiver
CREATE INDEX idx_friend_request_accepted_receiver
    ON business.friend_request(receiver_id, status)
    WHERE status = 'accepted';