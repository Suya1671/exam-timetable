import { createTauRPCProxy as createProxy } from './generated'
export type * from './generated'

type ProxyInstance = ReturnType<typeof createProxy>

export const createTauRPCProxy = (): ProxyInstance => createProxy()
