// Authentication types for institutional portal

export enum UserRole {
  ADMIN = 'ADMIN',
  TREASURY = 'TREASURY',
  COMPLIANCE = 'COMPLIANCE',
  VIEWER = 'VIEWER',
}

export enum KYCStatus {
  NOT_STARTED = 'NOT_STARTED',
  IN_PROGRESS = 'IN_PROGRESS',
  PENDING_REVIEW = 'PENDING_REVIEW',
  APPROVED = 'APPROVED',
  REJECTED = 'REJECTED',
}

export interface User {
  id: string;
  email: string;
  role: UserRole;
  organization: string;
  kycStatus: KYCStatus;
  walletAddress?: string;
  createdAt: Date;
  lastLoginAt: Date;
}

export interface Organization {
  id: string;
  legalName: string;
  registrationNumber: string;
  jurisdiction: string;
  entityType: EntityType;
  kycStatus: KYCStatus;
  approvedAt?: Date;
  treasury: {
    totalDeposited: string;
    totalMinted: string;
    activeCurrencies: string[];
  };
}

export enum EntityType {
  CORPORATION = 'CORPORATION',
  LLC = 'LLC',
  TRUST = 'TRUST',
  PARTNERSHIP = 'PARTNERSHIP',
  SOVEREIGN = 'SOVEREIGN',
  FINANCIAL_INSTITUTION = 'FINANCIAL_INSTITUTION',
}

export interface Session {
  user: User;
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface RegisterData {
  email: string;
  password: string;
  organizationName: string;
  role: UserRole;
}

