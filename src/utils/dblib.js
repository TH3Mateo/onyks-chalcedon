// Updates an Altium .DbLib file so that it covers every category table and supplier
// column of the database, without touching what the user configured in Altium.
//
// Kept from the existing file: the [DatabaseLinks] section (connection string, schema,
// quoting...), every [TableN] section of a category that still exists (with Altium's
// BrowserOrder_* state, Enabled, UserWhere) and every field map this program does not
// manage. Generated: a [TableN] section for each new category and the managed field
// maps of every table, modelled on the hand-configured "fpga" table.

// FieldMaps each category table gets, in this order.
const BASE_FIELDS = [
    { field: 'part_name', type: 0, parameter: 'part_name' },
    { field: 'description', type: 1, parameter: '[Description]' },
    { field: 'library_ref', type: 1, parameter: '[Library Ref]' },
    { field: 'library_path', type: 1, parameter: '[Library Path]' },
    { field: 'footprint_reference_1', type: 1, parameter: '[Footprint Ref]' },
    { field: 'footprint_path_1', type: 1, parameter: '[Footprint Path]' },
    { field: 'footprint_reference_2', type: 1, parameter: '[Footprint Ref 2]' },
    { field: 'footprint_path_2', type: 1, parameter: '[Footprint Path 2]' },
    { field: 'footprint_reference_3', type: 1, parameter: '[Footprint Ref 3]' },
    { field: 'footprint_path_3', type: 1, parameter: '[Footprint Path 3]' },
]

// Supplier code columns are all managed: one that is not current any more belongs to a
// deleted or renamed supplier, and its map is dropped.
const SUPPLIER_COLUMN = /^supplier_\d+_/

// Used only when the file does not exist yet.
const DEFAULT_HEADER = [
    { name: 'OutputDatabaseLinkFile', lines: ['Version=1.1'] },
    {
        name: 'DatabaseLinks',
        lines: [
            'ConnectionString=Provider=MSDASQL.1;Persist Security Info=False;Data Source=bloodstone',
            'AddMode=3',
            'RemoveMode=1',
            'UpdateMode=2',
            'ViewMode=0',
            'LeftQuote="',
            'RightQuote="',
            'QuoteTableNames=1',
            'UseTableSchemaName=0',
            'DefaultColumnType=VARCHAR(255)',
            'LibraryDatabaseType=',
            'LibraryDatabasePath=',
            'DatabasePathRelative=0',
            'TopPanelCollapsed=0',
            'LibrarySearchPath=',
            'OrcadMultiValueDelimiter=,',
            'SearchSubDirectories=0',
            'SchemaName=ALTIUM',
            'LastFocusedTable=',
        ],
    },
]

// Same convention as getSupplierColumnMap() in the Bloodstone backend (utils.py).
export function supplierColumnName(supplier)
{
    return `supplier_${supplier.id}_${supplier.name.toLowerCase().replace(/ /g, '_').replace(/-/g, '_')}`
}

export function normalize(text)
{
    return text.replace(/\r\n/g, '\n').replace(/\n+$/, '')
}

function parse(text)
{
    const sections = []
    for (const line of normalize(text).split('\n'))
    {
        const header = line.match(/^\[(.+)\]$/)
        if (header)
        {
            sections.push({ name: header[1], lines: [] })
        }
        else if (sections.length && line !== '')
        {
            sections[sections.length - 1].lines.push(line)
        }
    }
    return sections
}

function value(section, key)
{
    const line = section.lines.find((l) => l.startsWith(`${key}=`))
    return line === undefined ? null : line.slice(key.length + 1)
}

function setValue(section, key, newValue)
{
    const index = section.lines.findIndex((l) => l.startsWith(`${key}=`))
    if (index >= 0)
    {
        section.lines[index] = `${key}=${newValue}`
    }
}

// "Options=FieldName=t.f|TableNameOnly=t|FieldNameOnly=f|..." -> {TableNameOnly: 't', ...}
function fieldMapOptions(section)
{
    const options = {}
    for (const part of (value(section, 'Options') ?? '').split('|'))
    {
        const separator = part.indexOf('=')
        if (separator > 0)
        {
            options[part.slice(0, separator)] = part.slice(separator + 1)
        }
    }
    return options
}

function fieldMapLine({ table, field, type, parameter })
{
    return `Options=FieldName=${table}.${field}|TableNameOnly=${table}|FieldNameOnly=${field}`
        + `|FieldType=${type}|ParameterName=${parameter}`
        + '|VisibleOnAdd=False|AddMode=0|RemoveMode=0|UpdateMode=0'
}

// `existing` is the current file content or null; `tables` the category names in the
// order new ones should be appended; `suppliers` a list of {id, name}.
export function buildDbLib(existing, tables, suppliers)
{
    const sections = existing === null ? [] : parse(existing)
    const isTable = (s) => /^Table\d+$/.test(s.name)
    const isFieldMap = (s) => /^FieldMap\d+$/.test(s.name)

    let header = sections.filter((s) => !isTable(s) && !isFieldMap(s))
    if (!header.some((s) => s.name === 'DatabaseLinks'))
    {
        header = [...structuredClone(DEFAULT_HEADER), ...header]
    }

    const current = new Set(tables)

    const tableSections = sections
        .filter(isTable)
        .filter((s) => current.has(value(s, 'TableName')))
    const known = new Set(tableSections.map((s) => value(s, 'TableName')))
    for (const table of tables.filter((t) => !known.has(t)))
    {
        tableSections.push({
            name: 'Table',
            lines: ['SchemaName=', `TableName=${table}`, 'Enabled=True', 'UserWhere=0', 'UserWhereText='],
        })
    }
    const orderedTables = tableSections.map((s) => value(s, 'TableName'))

    const managedFields = [
        ...BASE_FIELDS,
        ...suppliers.map((s) => ({ field: supplierColumnName(s), type: 1, parameter: s.name })),
    ]
    const managedNames = new Set(managedFields.map((f) => f.field))
    const isManaged = (field) => managedNames.has(field) || SUPPLIER_COLUMN.test(field)

    const keptFieldMaps = sections
        .filter(isFieldMap)
        .map((s) => ({ section: s, options: fieldMapOptions(s) }))
        .filter(({ options }) => current.has(options.TableNameOnly) && !isManaged(options.FieldNameOnly))

    // One entry per [FieldMapN] section, as its list of lines.
    const fieldMaps = []
    for (const table of orderedTables)
    {
        for (const field of managedFields)
        {
            fieldMaps.push([fieldMapLine({ table, ...field })])
        }
        for (const { section, options } of keptFieldMaps)
        {
            if (options.TableNameOnly === table)
            {
                fieldMaps.push(section.lines)
            }
        }
    }

    const links = header.find((s) => s.name === 'DatabaseLinks')
    const lastFocused = value(links, 'LastFocusedTable')
    if (lastFocused !== null && !current.has(lastFocused))
    {
        setValue(links, 'LastFocusedTable', orderedTables[0] ?? '')
    }

    const out = []
    for (const section of header)
    {
        out.push(`[${section.name}]`, ...section.lines)
    }
    tableSections.forEach((section, index) =>
    {
        out.push(`[Table${index + 1}]`, ...section.lines)
    })
    fieldMaps.forEach((lines, index) =>
    {
        out.push(`[FieldMap${index + 1}]`, ...lines)
    })

    // Altium itself saves the file with Windows line endings.
    return out.join('\r\n') + '\r\n'
}
