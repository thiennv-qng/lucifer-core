import { Address, AnchorProvider, Provider, web3 } from '@project-serum/anchor'
import { Connection, PublicKey, Transaction } from '@solana/web3.js'
import { WalletInterface } from './rawWallet'

/**
 * Validate an address
 * @param address Base58 string
 * @returns true/false
 */
export const isAddress = (address?: Address): address is Address => {
  if (!address) return false
  try {
    const publicKey = new web3.PublicKey(address)
    if (!publicKey) throw new Error('Invalid public key')
    return true
  } catch (er) {
    return false
  }
}

export const getAnchorProvider = (
  node: string,
  walletAddress: string,
  wallet: WalletInterface,
): Provider => {
  const connection = new Connection(node, 'confirmed')

  const signAllTransactions = async (transactions: Transaction[]) => {
    const signedTransactions = []
    for (const transaction of transactions) {
      const signedTransaction = await wallet.signTransaction(transaction)
      signedTransactions.push(signedTransaction)
    }
    return signedTransactions
  }

  const publicKey = new PublicKey(walletAddress)
  return new AnchorProvider(
    connection,
    {
      publicKey: new PublicKey(publicKey),
      signTransaction: wallet.signTransaction,
      signAllTransactions,
    },
    {
      commitment: 'confirmed',
      skipPreflight: true,
    },
  )
}
