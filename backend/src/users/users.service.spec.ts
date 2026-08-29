import { NotFoundException } from '@nestjs/common';
import { UserRole } from '@prisma/client';
import { PrismaService } from '../prisma/prisma.service';
import { UsersService } from './users.service';

describe('UsersService', () => {
  let service: UsersService;
  let prisma: {
    user: { findUnique: jest.Mock; update: jest.Mock };
  };

  const earner = {
    id: 'user-1',
    address: 'GABC',
    role: UserRole.EARNER,
  };

  beforeEach(() => {
    prisma = {
      user: { findUnique: jest.fn(), update: jest.fn() },
    };

    service = new UsersService(prisma as unknown as PrismaService);
  });

  it('returns the profile for the authenticated address', async () => {
    prisma.user.findUnique.mockResolvedValue(earner);

    await expect(service.getByAddress('GABC')).resolves.toEqual(earner);
    expect(prisma.user.findUnique).toHaveBeenCalledWith({
      where: { address: 'GABC' },
    });
  });

  it('reports a missing user rather than returning null', async () => {
    prisma.user.findUnique.mockResolvedValue(null);

    await expect(service.getByAddress('GABC')).rejects.toThrow(
      NotFoundException,
    );
  });

  it('persists a changed role against the address from the token', async () => {
    prisma.user.findUnique.mockResolvedValue(earner);
    prisma.user.update.mockResolvedValue({ ...earner, role: UserRole.CREATOR });

    await expect(service.setRole('GABC', UserRole.CREATOR)).resolves.toEqual({
      ...earner,
      role: UserRole.CREATOR,
    });
    expect(prisma.user.update).toHaveBeenCalledWith({
      where: { address: 'GABC' },
      data: { role: UserRole.CREATOR },
    });
  });

  it('does not write when the role is already the one requested', async () => {
    prisma.user.findUnique.mockResolvedValue(earner);

    await expect(service.setRole('GABC', UserRole.EARNER)).resolves.toEqual(
      earner,
    );
    expect(prisma.user.update).not.toHaveBeenCalled();
  });

  it('refuses to create a user from a token whose user is gone', async () => {
    prisma.user.findUnique.mockResolvedValue(null);

    await expect(service.setRole('GABC', UserRole.CREATOR)).rejects.toThrow(
      NotFoundException,
    );
    expect(prisma.user.update).not.toHaveBeenCalled();
  });
});
