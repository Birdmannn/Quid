-- CreateEnum
CREATE TYPE "UserRole" AS ENUM ('EARNER', 'CREATOR');

-- CreateEnum
CREATE TYPE "MissionStatus" AS ENUM ('OPEN', 'STARTED', 'PAUSED', 'COMPLETED', 'CANCELLED');

-- CreateEnum
CREATE TYPE "SubmissionStatus" AS ENUM ('PENDING', 'APPROVED', 'REJECTED', 'PAID');

-- CreateTable
CREATE TABLE "users" (
    "id" TEXT NOT NULL,
    "address" TEXT NOT NULL,
    "email" TEXT,
    "display_name" TEXT,
    "bio" TEXT,
    "role" "UserRole" NOT NULL DEFAULT 'EARNER',
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "users_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "missions" (
    "id" TEXT NOT NULL,
    "owner_address" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "description_cid" TEXT NOT NULL,
    "metadata_cid" TEXT NOT NULL,
    "metadata" JSONB NOT NULL,
    "reward_token" TEXT NOT NULL,
    "reward_amount" TEXT NOT NULL,
    "max_participants" INTEGER NOT NULL DEFAULT 0,
    "participants_count" INTEGER NOT NULL DEFAULT 0,
    "status" "MissionStatus" NOT NULL DEFAULT 'OPEN',
    "ai_summary" TEXT NOT NULL,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "missions_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "submissions" (
    "id" TEXT NOT NULL,
    "mission_id" TEXT NOT NULL,
    "hunter_address" TEXT NOT NULL,
    "ipfs_cid" TEXT NOT NULL,
    "text_payload" TEXT,
    "sentiment" DOUBLE PRECISION,
    "status" "SubmissionStatus" NOT NULL DEFAULT 'PENDING',
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "submissions_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "indexer_state" (
    "id" INTEGER NOT NULL DEFAULT 1,
    "last_ledger" BIGINT NOT NULL,
    "last_cursor" TEXT,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "indexer_state_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "users_address_key" ON "users"("address");

-- CreateIndex
CREATE UNIQUE INDEX "users_email_key" ON "users"("email");

-- CreateIndex
CREATE INDEX "missions_owner_address_idx" ON "missions"("owner_address");

-- CreateIndex
CREATE INDEX "missions_status_created_at_idx" ON "missions"("status", "created_at" DESC);

-- CreateIndex
CREATE INDEX "submissions_hunter_address_idx" ON "submissions"("hunter_address");

-- CreateIndex
CREATE INDEX "submissions_mission_id_idx" ON "submissions"("mission_id");

-- CreateIndex
CREATE UNIQUE INDEX "submissions_mission_id_hunter_address_key" ON "submissions"("mission_id", "hunter_address");

-- AddForeignKey
ALTER TABLE "missions" ADD CONSTRAINT "missions_owner_address_fkey" FOREIGN KEY ("owner_address") REFERENCES "users"("address") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "submissions" ADD CONSTRAINT "submissions_mission_id_fkey" FOREIGN KEY ("mission_id") REFERENCES "missions"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "submissions" ADD CONSTRAINT "submissions_hunter_address_fkey" FOREIGN KEY ("hunter_address") REFERENCES "users"("address") ON DELETE RESTRICT ON UPDATE CASCADE;
