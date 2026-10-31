// Builds the KiCad .kicad_dbl file from what the server generates (GET /api/settings/kicad-dbl:
// libraries, fields and supplier columns follow the database).
//
// The file never contains credentials, just like the Altium .DbLib: the connection and the
// login come from the PostgreSQL ODBC driver (its data source), so no password can end up
// in the file or in the repository. `username` and `password` are always written empty -
// also when someone typed them into an existing file. Only the name of an ODBC data
// source (`dsn`, not a secret) chosen in the existing file is kept; otherwise it is the
// "bloodstone" source that the program creates (composables/useOdbcSource.js).

export const DEFAULT_DSN = 'bloodstone'

export function parseKicadDbl(existing)
{
    if (typeof existing !== 'string')
    {
        return null
    }
    try
    {
        const parsed = JSON.parse(existing.replace(/^﻿/, ''))
        return parsed !== null && typeof parsed === 'object' ? parsed : null
    }
    catch (e)
    {
        return null
    }
}

// `existing`: current file text or null, `generated`: object from the server.
export function buildKicadDbl(existing, generated)
{
    const previous = parseKicadDbl(existing)
    const result = JSON.parse(JSON.stringify(generated))
    result.source = result.source ?? {}

    // The data source made by the program (login and password inside) replaces the
    // server's connection string, which would have to carry the credentials itself.
    // A different data source named in the existing file is kept.
    const dsn = previous?.source?.dsn
    result.source.dsn = typeof dsn === 'string' && dsn !== '' ? dsn : DEFAULT_DSN
    delete result.source.connection_string
    result.source.username = ''
    result.source.password = ''

    return JSON.stringify(result, null, 2) + '\n'
}

// Same comparison as for the DbLib: line endings and trailing newlines do not count.
export function normalizeKicadDbl(text)
{
    return text.replace(/\r\n/g, '\n').replace(/\n+$/, '')
}
