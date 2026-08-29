import { Injectable } from '@nestjs/common';
import { UploadedFilePayload } from './upload.types';
import * as crypto from 'crypto';

@Injectable()
export class UploadService {
  uploadFile(file: UploadedFilePayload) {
    const hash = file?.buffer
      ? crypto.createHash('sha256').update(file.buffer).digest('hex')
      : crypto
          .createHash('sha256')
          .update(file?.originalname || 'file')
          .digest('hex');
    const cid = `bafkrei${hash.slice(0, 48)}`;

    return {
      cid,
      filename: file?.originalname,
      mimeType: file?.mimetype,
      size: file?.size,
      'buffer-received': true,
    };
  }

  uploadJson(payload: any) {
    const jsonStr = JSON.stringify(payload ?? {});
    const hash = crypto.createHash('sha256').update(jsonStr).digest('hex');
    const cid = `bafkrei${hash.slice(0, 48)}`;

    return {
      cid,
      'json-received': true,
      'byte-length': Buffer.byteLength(jsonStr),
    };
  }
}
