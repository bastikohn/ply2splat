# GitHub Pages Setup Instructions

This document provides instructions for enabling GitHub Pages for the ply2splat web application.

## Prerequisites

- Repository owner/admin access to `bastikohn/ply2splat`
- The PR with GitHub Pages configuration has been merged to `main`

## Steps to Enable GitHub Pages

### 1. Access Repository Settings

1. Go to https://github.com/bastikohn/ply2splat
2. Click on **Settings** (you need admin access)
3. Scroll down to the **Pages** section in the left sidebar

### 2. Configure GitHub Pages Source

1. Under **Source**, select **GitHub Actions** from the dropdown
2. The workflow `.github/workflows/deploy-gh-pages.yml` will automatically deploy the site

### 3. Wait for Initial Deployment

1. Go to the **Actions** tab
2. You should see a workflow run called "Deploy to GitHub Pages"
3. Wait for it to complete (usually takes 2-3 minutes)
4. Once complete, the site will be available at: **https://bastikohn.github.io/ply2splat/**

### 4. Verify Deployment

1. Visit https://bastikohn.github.io/ply2splat/
2. You should see the PLY to SPLAT converter web application
3. Test the functionality by uploading a PLY file

## Deployment Details

### Automatic Deployment

The site automatically deploys when:
- Changes are pushed to the `main` branch
- Changes are made to files in `www/ply2splat/` directory
- The workflow is manually triggered

### Manual Deployment

To manually trigger a deployment:
1. Go to **Actions** tab
2. Select **Deploy to GitHub Pages** workflow
3. Click **Run workflow**
4. Select the `main` branch
5. Click **Run workflow**

## Troubleshooting

### Site Not Loading

If the site doesn't load after deployment:

1. Check that GitHub Pages is enabled and set to use GitHub Actions
2. Verify the latest workflow run completed successfully
3. Wait a few minutes for GitHub's CDN to update
4. Try clearing your browser cache

### CORS or COOP/COEP Issues

The application requires Cross-Origin Isolation for SharedArrayBuffer support. This is handled by the `coi-serviceworker.js` file which sets the required headers.

If you encounter issues:
1. Check browser console for errors
2. Verify the service worker is registered (DevTools > Application > Service Workers)
3. Try in a different browser (Chrome, Firefox, Edge are supported)

### Build Failures

If the workflow fails:
1. Check the workflow logs in the Actions tab
2. Verify all dependencies are correctly specified in `package.json`
3. Ensure the WASM package `@ply2splat/native-wasm32-wasi` is published and accessible

## Configuration

### Base Path

The application is configured to use `/ply2splat/` as the base path, matching the repository name. This is set in the workflow via the `VITE_BASE_PATH` environment variable.

If you need to change this (e.g., for a custom domain):
1. Update the `VITE_BASE_PATH` in `.github/workflows/deploy-gh-pages.yml`
2. Update the base URL in documentation

### Custom Domain

To use a custom domain:
1. Follow GitHub's instructions: https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site
2. Update `VITE_BASE_PATH` to `/` in the workflow
3. Add a `CNAME` file to the `www/ply2splat/public/` directory

## Support

If you encounter issues that aren't covered here:
1. Check the GitHub Actions logs for detailed error messages
2. Review the Vite and React documentation
3. Open an issue in the repository
