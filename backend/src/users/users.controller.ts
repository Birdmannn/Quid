import { Body, Controller, Get, Patch, Req, UseGuards } from '@nestjs/common';
import { User } from '@prisma/client';
import { Request } from 'express';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { UpdateUserRoleDto } from './dto/update-user-role.dto';
import { UsersService } from './users.service';

interface AuthenticatedRequest extends Request {
  user: {
    userId: string;
    address: string;
  };
}

/**
 * Issue #331: both routes are keyed off the address in the SEP-10 JWT, never
 * off a body field, so one signed-in wallet can only ever read or change its
 * own role.
 */
@Controller('users')
@UseGuards(JwtAuthGuard)
export class UsersController {
  constructor(private readonly usersService: UsersService) {}

  @Get('me')
  me(@Req() req: AuthenticatedRequest): Promise<User> {
    return this.usersService.getByAddress(req.user.address);
  }

  @Patch('me/role')
  updateRole(
    @Body() dto: UpdateUserRoleDto,
    @Req() req: AuthenticatedRequest,
  ): Promise<User> {
    return this.usersService.setRole(req.user.address, dto.role);
  }
}
