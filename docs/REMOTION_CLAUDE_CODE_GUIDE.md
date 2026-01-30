# Remotion + Claude Code: Complete User Guide

> Create professional videos and stills programmatically using React and AI

## Table of Contents

1. [Overview](#overview)
2. [How It Works](#how-it-works)
3. [Prerequisites](#prerequisites)
4. [Installation & Setup](#installation--setup)
5. [Core Concepts](#core-concepts)
6. [Creating Videos](#creating-videos)
7. [Creating Still Images](#creating-still-images)
8. [API Reference](#api-reference)
9. [Slash Commands](#slash-commands)
10. [Pricing & Licensing](#pricing--licensing)
11. [Cost Optimization](#cost-optimization)
12. [Best Practices](#best-practices)
13. [Troubleshooting](#troubleshooting)

---

## Overview

**Remotion** is a framework for creating videos programmatically using React. It treats video like a web page where each frame is a React component rendered at a specific point in time. You write JSX, style with CSS, animate with interpolation functions, and render to MP4.

**Claude Code + Remotion** enables you to generate, modify, and render videos using natural language instead of writing React code manually. Claude writes the code that renders your video, giving you precision, editability, and the ability to generate thousands of variations from a single template.

### Key Benefits

- **Programmatic Control**: Every pixel is code-controlled
- **React Ecosystem**: Use any React library, npm package, or web technology
- **AI-Powered**: Describe videos in natural language; Claude handles the code
- **Template-Based**: Create reusable templates for batch generation
- **Version Control**: Videos are code, so they can be versioned with Git

---

## How It Works

```
┌─────────────────────────────────────────────────────────────────┐
│                        YOUR WORKFLOW                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   1. You describe the video → "Create a 10-second intro with    │
│      my logo fading in and company name sliding from left"       │
│                                                                  │
│   2. Claude Code generates → React components with Remotion      │
│      animations, sequences, and compositions                     │
│                                                                  │
│   3. Preview in browser → localhost:3000 with timeline controls  │
│                                                                  │
│   4. Render to MP4 → npx remotion render                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

Remotion provides a frame number and a blank canvas. Your React components receive the current frame number and render visuals accordingly. The framework then captures each frame and stitches them into a video file.

---

## Prerequisites

### Required

| Requirement | Version | Purpose |
|------------|---------|---------|
| Node.js | 20+ | Runtime environment |
| pnpm/npm/yarn | Latest | Package management |
| Claude Code | Latest | AI code generation |
| FFmpeg | Latest | Audio/video processing |

### Optional (for AI Features)

| Service | Purpose | Cost |
|---------|---------|------|
| Replicate API | Image/video generation | Pay-per-use |
| Deepgram API | Audio transcription | Pay-per-use |
| ElevenLabs API | AI voiceovers | Pay-per-use |

---

## Installation & Setup

### Method 1: Create New Remotion Project with Skills (Recommended)

```bash
# Step 1: Create a new Remotion project
npx create-video@latest my-video
cd my-video

# Step 2: Install Remotion Agent Skills for Claude Code
npx skills add remotion-dev/skills

# Step 3: Start Claude Code
claude

# Step 4: Start the development server
pnpm run dev
# or
npm run dev
```

### Method 2: Use Claude Remotion Kickstart Template

```bash
# Clone the template
git clone https://github.com/jhartquist/claude-remotion-kickstart.git
cd claude-remotion-kickstart

# Install dependencies
pnpm install

# Start development
pnpm run dev
```

### Method 3: Add to Existing React Project

```bash
# Add Remotion packages (use exact versions!)
npm install --save-exact remotion@4.0.396 @remotion/cli@4.0.396

# Optional: Add renderer for SSR
npm install --save-exact @remotion/renderer@4.0.396

# Optional: Add Lambda for serverless rendering
npm install --save-exact @remotion/lambda@4.0.396

# Optional: Add player for embedding
npm install --save-exact @remotion/player@4.0.396
```

### Verify Installation

```bash
# Check Remotion version
npx remotion --version

# Start the studio
npx remotion studio
```

---

## Core Concepts

### 1. Composition

A composition defines a renderable video with metadata:

```tsx
import { Composition } from 'remotion';

export const RemotionRoot = () => {
  return (
    <Composition
      id="MyVideo"           // Unique identifier for rendering
      component={MyVideo}    // React component to render
      durationInFrames={300} // 10 seconds at 30fps
      fps={30}               // Frame rate
      width={1920}           // Video width
      height={1080}          // Video height
    />
  );
};
```

### 2. useCurrentFrame

Get the current frame number (0-indexed):

```tsx
import { useCurrentFrame, useVideoConfig } from 'remotion';

const MyComponent = () => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();

  // Calculate progress (0 to 1)
  const progress = frame / durationInFrames;

  return <div style={{ opacity: progress }}>Fading in...</div>;
};
```

### 3. AbsoluteFill

Layer elements on top of each other:

```tsx
import { AbsoluteFill } from 'remotion';

const LayeredScene = () => {
  return (
    <AbsoluteFill>
      <Background />
      <AbsoluteFill>
        <ForegroundContent />
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
```

### 4. Sequence

Time-shift components to appear at specific frames:

```tsx
import { Sequence } from 'remotion';

const Timeline = () => {
  return (
    <>
      {/* Appears immediately */}
      <Sequence from={0} durationInFrames={60}>
        <IntroSlide />
      </Sequence>

      {/* Appears at frame 60 (2 seconds at 30fps) */}
      <Sequence from={60} durationInFrames={90}>
        <MainContent />
      </Sequence>

      {/* Appears at frame 150 */}
      <Sequence from={150}>
        <OutroSlide />
      </Sequence>
    </>
  );
};
```

### 5. Interpolation

Create smooth animations:

```tsx
import { useCurrentFrame, interpolate, spring } from 'remotion';

const AnimatedBox = () => {
  const frame = useCurrentFrame();

  // Linear interpolation
  const opacity = interpolate(frame, [0, 30], [0, 1]);

  // Spring animation
  const scale = spring({
    frame,
    fps: 30,
    config: { damping: 10, stiffness: 100 }
  });

  return (
    <div style={{
      opacity,
      transform: `scale(${scale})`
    }}>
      Animated!
    </div>
  );
};
```

---

## Creating Videos

### Basic Workflow

1. **Describe your video to Claude Code:**
   ```
   Create a 15-second product showcase video with:
   - Animated title "New Product Launch"
   - Product image sliding in from right
   - Feature bullets appearing one by one
   - Call-to-action at the end
   ```

2. **Preview in browser:**
   ```bash
   pnpm run dev
   # Opens localhost:3000 with timeline controls
   ```

3. **Render to video file:**
   ```bash
   # Render specific composition
   npx remotion render MyVideo out/MyVideo.mp4

   # Render with options
   npx remotion render MyVideo out/MyVideo.mp4 --codec=h264 --quality=80
   ```

### Render Options

| Option | Values | Description |
|--------|--------|-------------|
| `--codec` | h264, h265, vp8, vp9, prores, gif | Video codec |
| `--quality` | 0-100 | Quality for lossy codecs |
| `--scale` | 0.5, 1, 1.5, 2 | Scale output dimensions |
| `--frames` | 0-100, 50-150 | Render specific frame range |
| `--concurrency` | 1-16 | Parallel renders |

### Video Presets

```bash
# 1080p (default)
npx remotion render MyVideo --width=1920 --height=1080

# 720p
npx remotion render MyVideo --width=1280 --height=720

# Square (Instagram)
npx remotion render MyVideo --width=1080 --height=1080

# Portrait (TikTok/Reels)
npx remotion render MyVideo --width=1080 --height=1920

# 4K
npx remotion render MyVideo --width=3840 --height=2160
```

---

## Creating Still Images

### Command Line

```bash
# Render first frame as PNG (default)
npx remotion still MyVideo out/thumbnail.png

# Render specific frame
npx remotion still MyVideo out/frame50.png --frame=50

# Render last frame
npx remotion still MyVideo out/last.png --frame=-1

# Render as JPEG with quality
npx remotion still MyVideo out/thumb.jpg --image-format=jpeg --quality=90

# Render as WebP
npx remotion still MyVideo out/thumb.webp --image-format=webp

# Scale output
npx remotion still MyVideo out/thumb.png --scale=2
```

### Node.js API

```typescript
import { renderStill, selectComposition, bundle } from '@remotion/renderer';

async function createThumbnail() {
  // Bundle the project
  const bundled = await bundle({
    entryPoint: './src/index.ts',
  });

  // Get composition info
  const composition = await selectComposition({
    serveUrl: bundled,
    id: 'MyVideo',
  });

  // Render still
  await renderStill({
    composition,
    serveUrl: bundled,
    output: './out/thumbnail.png',
    frame: 0,
    imageFormat: 'png',
  });
}
```

### Output Formats

| Format | Extension | Best For |
|--------|-----------|----------|
| PNG | .png | High quality, transparency |
| JPEG | .jpg | Photos, smaller files |
| WebP | .webp | Web optimization |
| PDF | .pdf | Print, documents |

---

## API Reference

### @remotion/renderer

#### renderMedia()

Render a video or audio file:

```typescript
import { renderMedia, selectComposition, bundle } from '@remotion/renderer';

const bundled = await bundle({ entryPoint: './src/index.ts' });

const composition = await selectComposition({
  serveUrl: bundled,
  id: 'MyVideo',
});

await renderMedia({
  composition,
  serveUrl: bundled,
  codec: 'h264',
  outputLocation: './out/video.mp4',
  onProgress: ({ progress }) => {
    console.log(`Rendering: ${Math.round(progress * 100)}%`);
  },
});
```

#### renderStill()

Render a single frame:

```typescript
import { renderStill } from '@remotion/renderer';

await renderStill({
  composition,
  serveUrl: bundled,
  output: './out/frame.png',
  frame: 0,
  imageFormat: 'png',
  scale: 1,
});
```

### @remotion/lambda

#### renderMediaOnLambda()

Trigger serverless video render:

```typescript
import { renderMediaOnLambda } from '@remotion/lambda';

const { bucketName, renderId } = await renderMediaOnLambda({
  region: 'us-east-1',
  functionName: 'remotion-render',
  composition: 'MyVideo',
  serveUrl: 'https://your-bucket.s3.amazonaws.com/bundle/index.html',
  codec: 'h264',
});
```

#### renderStillOnLambda()

Trigger serverless still render:

```typescript
import { renderStillOnLambda } from '@remotion/lambda';

const { url } = await renderStillOnLambda({
  region: 'us-east-1',
  functionName: 'remotion-render',
  composition: 'MyVideo',
  serveUrl: bundleUrl,
  imageFormat: 'png',
  frame: 0,
});
```

---

## Slash Commands

When using the Claude Remotion Kickstart template, these slash commands are available:

| Command | Function |
|---------|----------|
| `/new-composition <name>` | Create boilerplate video project |
| `/generate-image <prompt>` | AI image creation (requires Replicate API) |
| `/generate-video <prompt>` | AI video generation (requires Replicate API) |
| `/transcribe <file>` | Audio transcription (requires Deepgram API) |
| `/screenshot <url>` | Full-page screenshot capture |

### Usage Examples

```
/new-composition product-demo
→ Creates src/compositions/product-demo/ with boilerplate

/generate-image "futuristic city skyline at sunset"
→ Generates image and saves to public/images/

/transcribe public/audio/narration.mp3
→ Creates word-level timestamps for caption sync
```

---

## Pricing & Licensing

### Remotion License Tiers

#### Free License (No Cost)

You are eligible to use Remotion for **free** if you are:

| Eligible Entity | Commercial Use? |
|-----------------|-----------------|
| Individual developers | Yes |
| For-profit organizations with **≤3 employees** | Yes |
| Non-profit / not-for-profit organizations | Yes |
| Evaluating Remotion (not commercial yet) | No |

#### Company License (Required for 4+ employees)

| Plan | Price | Billing |
|------|-------|---------|
| Developer License | $25/month or $250/year | Per developer |
| Company License | ~$100/month minimum | Per company |
| Enterprise | Custom pricing | Annual |

**Note:** For collaborative engagements (agencies + clients), the total employee count across all parties is aggregated. Part-time employees and contractors count toward the total.

### AWS Lambda Costs (If Using Serverless Rendering)

Lambda rendering incurs AWS charges in addition to Remotion licensing:

| Cost Component | Description |
|----------------|-------------|
| Lambda Compute | Primary cost; based on region, memory, duration |
| S3 Storage | Storing bundles and rendered files |
| S3 Bandwidth | Transferring assets and output files |
| CloudWatch Logs | Default logging (can be disabled) |

#### Example Costs

| Scenario | Approximate Cost |
|----------|------------------|
| HelloWorld (warm Lambda) | ~$0.001 |
| HelloWorld (cold Lambda) | ~$0.001 |
| 1-minute video (typical) | ~$0.01 - $0.05 |
| Complex 5-minute video | ~$0.10 - $0.50 |

**Most users render multiple minutes of video for just a few pennies.**

### Third-Party API Costs (Optional Features)

| Service | Use Case | Pricing |
|---------|----------|---------|
| Replicate | AI image/video generation | Pay-per-use (~$0.01-0.50/generation) |
| Deepgram | Audio transcription | Pay-per-minute (~$0.0043/min) |
| ElevenLabs | AI voiceovers | Pay-per-character or subscription |

### Cost Summary Table

| You Are | Remotion License | Lambda Costs | Total Monthly* |
|---------|------------------|--------------|----------------|
| Individual hobbyist | Free | Local rendering: $0 | **$0** |
| Solo freelancer | Free | ~$1-10 | **$1-10** |
| Startup (3 people) | Free | ~$10-50 | **$10-50** |
| Small company (4-10) | $100+/month | ~$20-100 | **$120-200** |
| Enterprise | Custom | Variable | **Custom** |

*Estimates based on moderate usage; actual costs depend on render volume and complexity.

---

## Cost Optimization

### Local Rendering (Zero Lambda Cost)

```bash
# Render locally instead of Lambda
npx remotion render MyVideo out/video.mp4

# Use fewer concurrent renders to reduce memory
npx remotion render MyVideo --concurrency=2
```

### Lambda Optimization

1. **Choose cheaper AWS regions** (check AWS Lambda pricing)
2. **Use warm Lambdas** (keep functions active)
3. **Reduce parallelization** for lower overhead
4. **Optimize asset sizes** (compress images, use efficient formats)
5. **Disable CloudWatch logs** if not needed

### Price Estimation Function

```typescript
import { estimatePrice } from '@remotion/lambda';

const price = estimatePrice({
  region: 'us-east-1',
  durationInMilliseconds: 10000, // 10 seconds of Lambda time
  memorySizeInMb: 2048,
  architecture: 'arm64', // Cheaper than x86
});

console.log(`Estimated cost: $${price.toFixed(4)}`);
```

---

## Best Practices

### Code Organization

```
src/
├── compositions/          # Video compositions
│   ├── my-video/
│   │   ├── index.tsx      # Main composition
│   │   ├── content.ts     # Content/data
│   │   └── scenes/        # Scene components
│   └── another-video/
├── components/            # Reusable components
│   ├── TitleSlide.tsx
│   ├── CodeBlock.tsx
│   └── Caption.tsx
├── hooks/                 # Custom hooks
├── utils/                 # Helper functions
└── Root.tsx               # Composition registry
```

### Performance Tips

1. **Memoize expensive calculations**
   ```tsx
   const expensiveValue = useMemo(() => calculate(frame), [frame]);
   ```

2. **Avoid re-renders**
   ```tsx
   // Use staticFile() for assets
   import { staticFile } from 'remotion';
   const logo = staticFile('logo.png');
   ```

3. **Preload assets**
   ```tsx
   import { prefetch } from 'remotion';
   prefetch('https://example.com/video.mp4');
   ```

### Claude Code Tips

1. **Be specific in descriptions**
   - Bad: "Make a cool intro"
   - Good: "Create a 5-second intro with white text 'ACME Corp' centered on black background, fading in over 1 second, holding for 3 seconds, then fading out"

2. **Iterate incrementally**
   - Start simple, add complexity gradually
   - Preview after each change

3. **Use consistent naming**
   - Composition IDs: PascalCase (MyVideo)
   - Files: kebab-case (my-video.tsx)

---

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| "Composition not found" | Check ID matches exactly (case-sensitive) |
| "FFmpeg not found" | Install FFmpeg: `brew install ffmpeg` or `apt install ffmpeg` |
| Version mismatch | Ensure all @remotion/* packages have same version |
| Out of memory | Reduce `--concurrency` or scale down resolution |
| Skills not working | Re-run `npx skills add remotion-dev/skills` |

### Verify Skills Installation

```bash
# Check if skills are installed
ls -la .claude/skills/remotion/

# Should see SKILL.md file
```

### Reset Remotion

```bash
# Clear cache
rm -rf node_modules/.cache/remotion

# Reinstall dependencies
rm -rf node_modules
pnpm install
```

---

## Quick Reference

### Essential Commands

```bash
# Development
pnpm run dev              # Start Remotion Studio
npx remotion studio       # Alternative studio command

# Rendering Videos
npx remotion render <id>                    # Render to default location
npx remotion render <id> out/video.mp4      # Render to specific path
npx remotion render <id> --codec=h265       # Use specific codec

# Rendering Stills
npx remotion still <id>                     # Render frame 0
npx remotion still <id> --frame=50          # Render specific frame
npx remotion still <id> --image-format=jpeg # Output as JPEG

# Lambda
npx remotion lambda render <url> <id>       # Render on Lambda
npx remotion lambda still <url> <id>        # Still on Lambda

# Utilities
npx remotion upgrade                        # Upgrade Remotion
npx remotion --help                         # Show all commands
```

### Useful Links

- [Remotion Documentation](https://www.remotion.dev/docs/)
- [Remotion GitHub](https://github.com/remotion-dev/remotion)
- [Claude Remotion Kickstart](https://github.com/jhartquist/claude-remotion-kickstart)
- [Remotion License Info](https://www.remotion.dev/docs/license)
- [Lambda Cost Examples](https://www.remotion.dev/docs/lambda/cost-example)
- [API Reference](https://www.remotion.dev/docs/api)

---

## Summary

Remotion + Claude Code provides a powerful combination for programmatic video and image generation:

1. **Free for individuals and small teams** (≤3 employees)
2. **Local rendering is completely free** (no cloud costs)
3. **Lambda rendering costs pennies** per video for most use cases
4. **Company license required** for organizations with 4+ employees (~$100/month)
5. **One-time setup**, then generate videos through conversation

The initial setup takes about an hour. After that, you can generate professional videos, thumbnails, and images by simply describing what you want to Claude Code.

---

*Last updated: January 2026*
