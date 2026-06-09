import { useEffect, useState, useRef, useMemo } from "react";
import { useNavigate, useLocation } from "react-router-dom";

// Characters used for the matrix rain effect
const MATRIX_CHARS = "01アイウエオカキクケコサシスセソタチツテト";

function MatrixRain() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };
    resize();
    window.addEventListener("resize", resize);

    const fontSize = 14;
    const columns = Math.floor(canvas.width / fontSize);
    const drops: number[] = Array.from({ length: columns }, () =>
      Math.random() * -100
    );

    const draw = () => {
      ctx.fillStyle = "rgba(0, 0, 0, 0.06)";
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = "#22c55e";
      ctx.font = `${fontSize}px monospace`;

      drops.forEach((y, i) => {
        const char = MATRIX_CHARS[Math.floor(Math.random() * MATRIX_CHARS.length)];
        const opacity = 0.15 + Math.random() * 0.35;
        ctx.fillStyle = `rgba(34, 197, 94, ${opacity})`;
        ctx.fillText(char, i * fontSize, y * fontSize);

        if (y * fontSize > canvas.height && Math.random() > 0.975) {
          drops[i] = 0;
        }
        drops[i] += 0.5 + Math.random() * 0.5;
      });
    };

    const interval = setInterval(draw, 45);
    return () => {
      clearInterval(interval);
      window.removeEventListener("resize", resize);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 0,
        pointerEvents: "none",
      }}
    />
  );
}

export default function NotFoundPage() {
  const navigate = useNavigate();
  const location = useLocation();
  const [typedLines, setTypedLines] = useState<string[]>([]);
  const [cursorVisible, setCursorVisible] = useState(true);

  const terminalLines = useMemo(() => [
    `$ rescope route ${location.pathname}`,
    "error: route not found in emulator scope",
    "",
    "  ╭─ E404 ──────────────────────╮",
    "  │                             │",
    "  │   Page not found.           │",
    "  │   This route doesn't exist  │",
    "  │   in the emulator.          │",
    "  │                             │",
    "  ╰─────────────────────────────╯",
    "",
    "hint: try `rescope route /dashboard`",
  // eslint-disable-next-line react-hooks/exhaustive-deps
  ], []);

  // Typewriter effect
  useEffect(() => {
    let lineIndex = 0;
    let cancelled = false;

    const addLine = () => {
      if (cancelled || lineIndex >= terminalLines.length) return;
      const line = terminalLines[lineIndex];
      setTypedLines((prev) => [...prev, line]);
      lineIndex++;
      setTimeout(addLine, lineIndex <= 1 ? 300 : 60 + Math.random() * 80);
    };

    setTimeout(addLine, 400);
    return () => { cancelled = true; };
  }, [terminalLines]);

  // Blinking cursor
  useEffect(() => {
    const id = setInterval(() => setCursorVisible((v) => !v), 530);
    return () => clearInterval(id);
  }, []);

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "#0a0a0a",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        fontFamily: "var(--font-mono, 'JetBrains Mono', monospace)",
        overflow: "hidden",
      }}
    >
      <MatrixRain />

      {/* Content container */}
      <div
        style={{
          position: "relative",
          zIndex: 1,
          maxWidth: 560,
          width: "100%",
          padding: "var(--space-6, 32px)",
        }}
      >
        {/* Glitching 404 */}
        <div
          style={{
            fontSize: 96,
            fontWeight: 900,
            letterSpacing: "-0.04em",
            color: "#22c55e",
            textShadow:
              "0 0 20px rgba(34,197,94,0.4), 0 0 60px rgba(34,197,94,0.15)",
            animation: "glitch 3s infinite",
            textAlign: "center",
            marginBottom: 32,
            lineHeight: 1,
          }}
        >
          404
        </div>

        {/* Terminal lines */}
        <div
          style={{
            background: "rgba(0, 0, 0, 0.7)",
            border: "1px solid rgba(34, 197, 94, 0.2)",
            borderRadius: 8,
            padding: "20px 24px",
            backdropFilter: "blur(8px)",
          }}
        >
          {typedLines.map((line, i) => (
            <div
              key={i}
              style={{
                fontSize: 13,
                lineHeight: 1.6,
                color: (line ?? "").startsWith("error")
                  ? "#ef4444"
                  : (line ?? "").startsWith("hint")
                  ? "#facc15"
                  : (line ?? "").startsWith("$")
                  ? "#22c55e"
                  : "#888",
                whiteSpace: "pre",
                animation: "fadeIn 0.15s ease-out",
              }}
            >
              {line}
            </div>
          ))}
          {typedLines.length < terminalLines.length && (
            <span
              style={{
                display: "inline-block",
                width: 8,
                height: 16,
                background: cursorVisible ? "#22c55e" : "transparent",
                verticalAlign: "text-bottom",
                marginLeft: 2,
              }}
            />
          )}
        </div>

        {/* Action button */}
        <div style={{ textAlign: "center", marginTop: 32 }}>
          <button
            onClick={() => navigate("/dashboard")}
            style={{
              background: "transparent",
              border: "1px solid rgba(34, 197, 94, 0.4)",
              color: "#22c55e",
              padding: "10px 28px",
              borderRadius: 6,
              fontSize: 14,
              fontFamily: "var(--font-mono, monospace)",
              cursor: "pointer",
              letterSpacing: "0.04em",
              transition: "all 0.2s",
              animation: "fadeIn 0.5s ease-out 1.5s both",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = "rgba(34, 197, 94, 0.1)";
              e.currentTarget.style.borderColor = "#22c55e";
              e.currentTarget.style.boxShadow =
                "0 0 15px rgba(34, 197, 94, 0.2)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = "transparent";
              e.currentTarget.style.borderColor = "rgba(34, 197, 94, 0.4)";
              e.currentTarget.style.boxShadow = "none";
            }}
          >
            cd ~/dashboard
          </button>
        </div>
      </div>

      <style>{`
        @keyframes glitch {
          0%, 90%, 100% { transform: translate(0); text-shadow: 0 0 20px rgba(34,197,94,0.4), 0 0 60px rgba(34,197,94,0.15); }
          92% { transform: translate(-3px, 1px); text-shadow: 3px 0 #ef4444, -3px 0 #3b82f6, 0 0 20px rgba(34,197,94,0.4); }
          94% { transform: translate(3px, -1px); text-shadow: -3px 0 #ef4444, 3px 0 #3b82f6, 0 0 20px rgba(34,197,94,0.4); }
          96% { transform: translate(-1px, 2px); text-shadow: 2px 0 #ef4444, -2px 0 #3b82f6, 0 0 20px rgba(34,197,94,0.4); }
          98% { transform: translate(1px, -1px); text-shadow: -1px 0 #ef4444, 1px 0 #3b82f6, 0 0 20px rgba(34,197,94,0.4); }
        }
        @keyframes fadeIn {
          from { opacity: 0; transform: translateY(4px); }
          to { opacity: 1; transform: translateY(0); }
        }
      `}</style>
    </div>
  );
}
