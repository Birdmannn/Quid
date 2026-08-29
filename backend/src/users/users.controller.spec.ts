import { GUARDS_METADATA } from '@nestjs/common/constants';
import { UserRole } from '@prisma/client';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { UsersController } from './users.controller';
import { UsersService } from './users.service';

describe('UsersController', () => {
  let controller: UsersController;
  let usersService: { getByAddress: jest.Mock; setRole: jest.Mock };

  const authenticated = {
    user: { userId: 'user-1', address: 'GABC' },
  };

  beforeEach(() => {
    usersService = { getByAddress: jest.fn(), setRole: jest.fn() };
    controller = new UsersController(usersService as unknown as UsersService);
  });

  it('returns the profile of the address in the token', async () => {
    const user = { id: 'user-1', address: 'GABC', role: UserRole.EARNER };
    usersService.getByAddress.mockResolvedValue(user);

    await expect(controller.me(authenticated as any)).resolves.toEqual(user);
    expect(usersService.getByAddress).toHaveBeenCalledWith('GABC');
  });

  it('updates the role for the address in the token, not one from the body', async () => {
    const updated = { id: 'user-1', address: 'GABC', role: UserRole.CREATOR };
    usersService.setRole.mockResolvedValue(updated);

    await expect(
      controller.updateRole({ role: UserRole.CREATOR }, authenticated as any),
    ).resolves.toEqual(updated);
    expect(usersService.setRole).toHaveBeenCalledWith('GABC', UserRole.CREATOR);
  });

  it('requires JWT authentication for every route', () => {
    const guards = Reflect.getMetadata(GUARDS_METADATA, UsersController);

    expect(guards).toContain(JwtAuthGuard);
  });
});
