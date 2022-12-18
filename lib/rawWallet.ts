import { PublicKey, Transaction } from '@solana/web3.js'

interface ProviderWallet {
  disconnect: () => void
}

type Signature = {
  publicKey: PublicKey
  signature: Buffer
}

type SignedMessage = {
  address: string // Base58 string
  signature: string // Hex string
  message: string // Utf8 string
}

export interface WalletInterface {
  /**
   * Any string that you can recognize your wallet type. For example: PrivateKey, Phantom, Sollet, ...
   */
  walletType: string

  /**
   * Wallet providers are varied from the original wallet (Coin98, Slope, ...).
   * Seems there is no single common standard, thus we only require `disconnect` method for the returned `provider`.
   * @return You can return anything from `getProvider` that respects to {@link https://descartesnetwork.github.io/sen-js/interfaces/Provider.html | Provider}
   */
  getProvider(): Promise<ProviderWallet>

  /**
   * Return wallet address
   * @returns Wallet address (base58)
   */
  getAddress(): Promise<string>

  /**
   * Call `rawSignTransaction` for signature and add to the input transaction
   * @param transaction - The transaction that needs to be signed
   * @returns The signed transaction
   */
  signTransaction(transaction: Transaction): Promise<Transaction>

  /**
   * Sign the input transaction and return signature
   * @param transaction - The transaction that needs to be signed
   * @returns {@link https://descartesnetwork.github.io/sen-js/modules.html#Signature | Signature}
   */
  rawSignTransaction(transaction: Transaction): Promise<Signature>

  /**
   * Sign a message and return a signed messaged
   * @param message - String needs to be signed
   * @returns {@link https://descartesnetwork.github.io/sen-js/modules.html#SignedMessage | SignedMessage}
   */
  signMessage(message: string): Promise<SignedMessage>

  /**
   * Verify a singed message
   * @param signature - Signature (`signedMessage.signature`)
   * @param message - The original message (or `signedMessage.message`)
   * @param address - Optional. The address that signed the message. If not provided, the `address` will be fetched by `this.getAddress()`.
   */
  verifySignature(
    signature: string,
    message: string,
    address?: string,
  ): Promise<boolean>

  /**
   * Call the `disconnect` method from `provider` returned by `getProvider`
   */
  disconnect(): Promise<void>
}
