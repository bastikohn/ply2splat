import { Link } from '@tanstack/react-router'
import { Github } from 'lucide-react'
import { Button } from './ui/button'

export default function Header() {
  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container mx-auto flex h-14 max-w-4xl items-center justify-between px-4">
        <Link to="/" className="flex items-center gap-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-primary-foreground font-bold text-sm">
            P2S
          </div>
          <span className="font-semibold">ply2splat</span>
        </Link>

        <nav className="flex items-center gap-2">
          <Button variant="ghost" size="icon" asChild>
            <a
              href="https://github.com/bastikohn/ply2splat"
              target="_blank"
              rel="noopener noreferrer"
              aria-label="GitHub repository"
            >
              <Github className="h-5 w-5" />
            </a>
          </Button>
        </nav>
      </div>
    </header>
  )
}
