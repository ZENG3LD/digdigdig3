# C2IntelFeeds Authentication

## Authentication Requirements

**Status**: ✅ None Required

C2IntelFeeds is a public GitHub repository. All feeds are accessible without authentication.

## Access Method

**Type**: Public HTTPS access to GitHub raw files
**Base URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/`

### Unauthenticated Access

```http
GET /drb-ra/C2IntelFeeds/master/feeds/IPC2s-30day.csv HTTP/1.1
Host: raw.githubusercontent.com
User-Agent: YourApp/1.0
```

**No headers required**:
- No API keys
- No authentication tokens
- No OAuth
- No registration

## Rate Limits

Unauthenticated access to GitHub raw files has rate limits enforced by IP address:

**Unauthenticated**: ~60 requests/hour per IP (historical limit, subject to change)
**Recent updates (2024-2025)**: GitHub has rolled out more aggressive rate limiting on raw.githubusercontent.com

See `tiers_and_limits.md` for detailed rate limit information.

## Optional GitHub Authentication (For Higher Limits)

While not required to access C2IntelFeeds, using a GitHub token can increase rate limits:

### Personal Access Token (PAT)

```http
GET /drb-ra/C2IntelFeeds/master/feeds/IPC2s-30day.csv HTTP/1.1
Host: raw.githubusercontent.com
Authorization: token ghp_YourPersonalAccessToken
User-Agent: YourApp/1.0
```

**Benefits**:
- Higher rate limits: ~5000 requests/hour
- More reliable access for high-frequency polling
- Avoids shared IP rate limiting issues

### GitHub API Alternative

Instead of raw files, use GitHub Contents API (authenticated):

```http
GET /repos/drb-ra/C2IntelFeeds/contents/feeds/IPC2s-30day.csv HTTP/1.1
Host: api.github.com
Authorization: token ghp_YourPersonalAccessToken
Accept: application/vnd.github.v3.raw
```

**Rate limits**:
- Unauthenticated: 60 requests/hour
- Authenticated: 5000 requests/hour

## Creating a GitHub Token (Optional)

If implementing authenticated access:

1. Go to https://github.com/settings/tokens
2. Generate new token (classic)
3. Select scopes: `public_repo` (or no scopes needed for public repos)
4. Use token in `Authorization: token <TOKEN>` header

**Note**: Read-only access to public repositories doesn't require specific scopes.

## Security Considerations

### Data Integrity

- GitHub serves files over HTTPS (TLS 1.2+)
- Content integrity guaranteed by HTTPS
- No additional signature verification provided by C2IntelFeeds project

### Recommended Practices

1. **Verify repository authenticity**: Ensure you're accessing the official repository
2. **Use HTTPS only**: Never use unencrypted HTTP
3. **Validate CSV format**: Parse defensively (see response_formats.md)
4. **Rate limit compliance**: Respect GitHub's rate limits
5. **Token security**: If using PAT, store securely (environment variables, secrets management)

## Repository Access

- **Repository**: https://github.com/drb-ra/C2IntelFeeds
- **License**: Check repository LICENSE file
- **Public**: Yes (no registration required)

## Summary

| Feature | Status |
|---------|--------|
| Authentication required | ❌ No |
| API key | ❌ Not applicable |
| OAuth | ❌ Not applicable |
| Rate limits | ✅ Yes (IP-based, unauthenticated) |
| Optional GitHub token | ✅ Recommended for high-frequency use |
| HTTPS required | ✅ Yes (enforced) |
| Cost | Free |
