export interface DatabaseUser {
  name: string;
  isSuper: boolean;
  canCreateDb: boolean;
  canCreateRole: boolean;
  canLogin: boolean;
}

export interface Privilege {
  schema: string;
  table: string;
  privilegeType: string;
}
