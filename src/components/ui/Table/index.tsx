/**
 * Table Component Module
 *
 * @module Table
 * @description
 * The Table component is a specialized component for displaying tables.
 * High-performance virtualized grid system with core input integration.
 *
 * @example
 * <Table
 *   data={[
 *     {
 *       id: '1',
 *       name: 'John Doe',
 *       age: 30,
 *       email: [EMAIL_ADDRESS]'
 *     }
 *   ]}
 *   columns={[
 *     {
 *       accessorKey: 'name',
 *       header: 'Name',
 *       cell: (row) => <span>{row.original.name}</span>
 *     },
 *     {
 *       accessorKey: 'age',
 *       header: 'Age',
 *       cell: (row) => <span>{row.original.age}</span>
 *     },
 *     {
 *       accessorKey: 'email',
 *       header: 'Email',
 *       cell: (row) => <span>{row.original.email}</span>
 *     }
 *   ]}
 *   class="custom-class"
 *   rowHeight={50}
 *   stickyHeader={true}
 *   sortKey="name"
 *   sortOrder="asc"
 *   selectedIds={['1']}
 *   onSort={(key, order) => console.log(key, order)}
 *   onColumnResize={(column, width) => console.log(column, width)}
 *   onColumnVisibilityChange={(column, visible) => console.log(column, visible)}
 *   onRowClick={(row) => console.log(row)}
 *   onRowDoubleClick={(row) => console.log(row)}
 *   onScroll={(scroll) => console.log(scroll)}
 *   onRowMount={(row) => console.log(row)}
 *   keyField="id"
 *   emptyMessage="No data"
 *   emptyDescription="No data available"
 *   emptyIcon="table"
 *   onVisibleItemsChange={(items) => console.log(items)}
 * >
 */
export { Table } from './Table';
export type { TableProps, Column, SortOrder } from './types';
