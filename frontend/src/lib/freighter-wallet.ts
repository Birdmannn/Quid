import {
  getAddress,
  isConnected,
  setAllowed,
  signTransaction,
} from '@stellar/freighter-api';
import { Networks } from '@stellar/stellar-sdk';

export const FREIGHTER_WALLET = {
  id: 'freighter',
  name: 'Freighter',
  icon: 'https://stellar.creit.tech/wallet-icons/freighter.png',
} as const;

export const NETWORK_PASSPHRASE = Networks.TESTNET;

export async function connectFreighter(): Promise<string> {
  const allowed = await setAllowed();
  if (!allowed) {
    throw new Error('Freighter connection was not approved');
  }

  const { address } = await getAddress();
  if (!address) {
    throw new Error('No Freighter address returned');
  }

  return address;
}

export async function getFreighterAddressIfConnected(): Promise<string | null> {
  const connected = await isConnected();
  if (!connected) {
    return null;
  }

  const { address } = await getAddress();
  return address || null;
}

export async function signFreighterTransaction(
  unsignedXdr: string,
  address: string,
  networkPassphrase: Networks = NETWORK_PASSPHRASE,
): Promise<string> {
  const { signedTxXdr } = await signTransaction(unsignedXdr, {
    networkPassphrase,
    address,
  });

  return signedTxXdr;
}
