import { Injectable, NotFoundException } from '@nestjs/common';
import { User, UserRole } from '@prisma/client';
import { PrismaService } from '../prisma/prisma.service';

@Injectable()
export class UsersService {
  constructor(private readonly prisma: PrismaService) {}

  /**
   * Issue #331: the profile a signed-in client reads its role from.
   */
  async getByAddress(address: string): Promise<User> {
    const user = await this.prisma.user.findUnique({ where: { address } });

    if (!user) {
      throw new NotFoundException(`User ${address} not found`);
    }

    return user;
  }

  /**
   * Issue #331: persist the onboarding choice against the authenticated
   * address, so the role survives a cleared localStorage or a new device.
   *
   * The row already exists - SEP-10 verification upserts it before the JWT is
   * issued - so a miss here means the token outlived its user and is a 404
   * rather than a silent create.
   */
  async setRole(address: string, role: UserRole): Promise<User> {
    const user = await this.prisma.user.findUnique({ where: { address } });

    if (!user) {
      throw new NotFoundException(`User ${address} not found`);
    }

    if (user.role === role) {
      return user;
    }

    return this.prisma.user.update({ where: { address }, data: { role } });
  }
}
