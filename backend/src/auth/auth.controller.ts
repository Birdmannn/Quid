import { Body, Controller, Get, Post, Query } from '@nestjs/common';
import { Throttle } from '@nestjs/throttler';
import { AuthService } from './auth.service';
import { VerifySignatureDto } from './dto/verify-signature.dto';

@Controller('auth')
export class AuthController {
  constructor(private readonly authService: AuthService) {}

  // Issue #348: stricter rate limit on SEP-10 challenge generation.
  // 10 requests per 60 s per IP to prevent enumeration / nonce farming.
  @Throttle({ default: { ttl: 60_000, limit: 10 } })
  @Get('challenge')
  generateChallenge(@Query('address') address: string) {
    return this.authService.generateChallenge(address);
  }

  // Issue #348: stricter rate limit on SEP-10 signature verification.
  // 5 requests per 60 s per IP to slow brute-force attempts.
  @Throttle({ default: { ttl: 60_000, limit: 5 } })
  @Post('verify')
  verify(@Body() dto: VerifySignatureDto) {
    return this.authService.verifySignedPayload(dto.signedXdr);
  }
}
