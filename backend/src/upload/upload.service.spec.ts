import { Test, TestingModule } from '@nestjs/testing';
import { UploadService } from './upload.service';

describe('UploadService', () => {
  let service: UploadService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [UploadService],
    }).compile();

    service = module.get<UploadService>(UploadService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  it('should upload JSON and return an IPFS CID', () => {
    const result = service.uploadJson({
      feedback: 'Great dApp UX!',
      rating: 5,
    });

    expect(result).toHaveProperty('cid');
    expect(result.cid).toMatch(/^bafkrei/);
    expect(result['json-received']).toBe(true);
  });

  it('should upload a file and return an IPFS CID', () => {
    const result = service.uploadFile({
      originalname: 'proof.png',
      mimetype: 'image/png',
      size: 1024,
      buffer: Buffer.from('mock-file-content'),
    });

    expect(result).toHaveProperty('cid');
    expect(result.cid).toMatch(/^bafkrei/);
    expect(result.filename).toBe('proof.png');
    expect(result['buffer-received']).toBe(true);
  });
});
