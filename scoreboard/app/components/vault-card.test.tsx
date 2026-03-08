import { render, screen } from '@testing-library/react'
import { VaultCard } from './vault-card'

// Mock @solana/react-hooks
jest.mock('@solana/react-hooks', () => ({
  useWalletConnection: jest.fn(),
  useSendTransaction: jest.fn(),
  useBalance: jest.fn(),
}))

// Mock @solana/kit
jest.mock('@solana/kit', () => ({
  getProgramDerivedAddress: jest.fn(),
  getAddressEncoder: jest.fn(() => ({ encode: jest.fn() })),
  getBytesEncoder: jest.fn(() => ({ encode: jest.fn() })),
}))

// Mock generated vault module
jest.mock('../generated/vault', () => ({
  getDepositInstructionDataEncoder: jest.fn(() => ({ encode: jest.fn() })),
  getWithdrawInstructionDataEncoder: jest.fn(() => ({ encode: jest.fn() })),
  VAULT_PROGRAM_ADDRESS: 'mock-program-address',
}))

const { useWalletConnection, useSendTransaction, useBalance } =
  jest.requireMock('@solana/react-hooks')

describe('VaultCard', () => {
  beforeEach(() => {
    useSendTransaction.mockReturnValue({ send: jest.fn(), isSending: false })
    useBalance.mockReturnValue(null)
  })

  it('shows "Wallet not connected" when wallet is disconnected', () => {
    useWalletConnection.mockReturnValue({ wallet: null, status: 'disconnected' })
    render(<VaultCard />)
    expect(screen.getByText('Wallet not connected')).toBeInTheDocument()
  })

  it('shows vault UI when wallet is connected', () => {
    useWalletConnection.mockReturnValue({
      wallet: { account: { address: 'mock-address' } },
      status: 'connected',
    })
    render(<VaultCard />)
    expect(screen.getByText('SOL Vault')).toBeInTheDocument()
    expect(screen.getByText('Vault Balance')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Amount in SOL')).toBeInTheDocument()
  })

  it('shows deposit and withdraw buttons when connected', () => {
    useWalletConnection.mockReturnValue({
      wallet: { account: { address: 'mock-address' } },
      status: 'connected',
    })
    render(<VaultCard />)
    expect(screen.getByRole('button', { name: /deposit/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /withdraw all/i })).toBeInTheDocument()
  })

  it('disables withdraw button when vault is empty', () => {
    useWalletConnection.mockReturnValue({
      wallet: { account: { address: 'mock-address' } },
      status: 'connected',
    })
    useBalance.mockReturnValue({ lamports: 0n })
    render(<VaultCard />)
    expect(screen.getByRole('button', { name: /withdraw all/i })).toBeDisabled()
  })
})
