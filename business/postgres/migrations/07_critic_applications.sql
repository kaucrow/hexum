CREATE TABLE platform.critic_application (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES platform.user(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at TIMESTAMPTZ,
    reviewed_by UUID REFERENCES platform.user(id)
);

-- Prevent duplicate pending applications from the same user
CREATE UNIQUE INDEX idx_critic_application_user_pending
    ON platform.critic_application (user_id)
    WHERE status = 'pending';