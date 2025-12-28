export interface Account {
  id: string;
  email: string;
  password: string;
  emailPassword: string;
  smtpServer: string;
  smtpPort: number;
  lastLoginTime?: string;
}

export interface BrowserSession {
  accountId: string;
  cookies?: string;
  localStorage?: string;
}

export interface EmailConfig {
  server: string;
  port: number;
  username: string;
  password: string;
  useSSL: boolean;
}

export interface VerificationCode {
  code: string;
  timestamp: number;
  from: string;
  subject: string;
}

export enum EmailStatus {
  Idle = 'idle',
  Connecting = 'connecting',
  Connected = 'connected',
  Receiving = 'receiving',
  Error = 'error',
  Stopped = 'stopped',
}

export interface EmailReceiverStatus {
  status: EmailStatus;
  errorMessage?: string;
  lastCheckTime?: number;
  codesCount: number;
}
