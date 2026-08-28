import { UserRole } from '@prisma/client';
import { IsEnum } from 'class-validator';

/**
 * Issue #331: the onboarding screen speaks 'creator' / 'hunter'; the database
 * speaks CREATOR / EARNER. The wire format is the Prisma enum, so the mapping
 * lives in exactly one place on the client (`lib/onboarding.ts`) and the API
 * rejects anything else outright.
 */
export class UpdateUserRoleDto {
  @IsEnum(UserRole)
  role!: UserRole;
}
