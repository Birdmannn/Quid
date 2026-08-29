import { Injectable, Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { ConfigService } from '@nestjs/config';
import { PrismaService } from '../prisma/prisma.service';

@Injectable()
export class IndexerService {
  private readonly logger = new Logger(IndexerService.name);

  constructor(
    private readonly config: ConfigService,
    private readonly prisma: PrismaService,
  ) {}

  // BE-047: 10-second cron scaffold
  @Cron(CronExpression.EVERY_10_SECONDS)
  async pollEvents(): Promise<void> {
    // BE-048: config gating — skip gracefully when chain config is absent
    const rpcUrl = this.config.get<string>('RPC_URL');
    const contractId = this.config.get<string>('CONTRACT_ID');

    if (!rpcUrl || !contractId) {
      this.logger.debug(
        'Skipping indexer tick: RPC_URL or CONTRACT_ID is not configured.',
      );
      return;
    }

    // BE-049: upsert singleton IndexerState checkpoint row
    await this.prisma.indexerState.upsert({
      where: { id: 1 },
      create: { id: 1, lastLedger: BigInt(0) },
      update: {},
    });

    this.logger.debug(
      'Indexer tick complete — IndexerState checkpoint upserted.',
    );
  }
}
