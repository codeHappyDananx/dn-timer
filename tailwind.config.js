import tailwindcssAnimate from "tailwindcss-animate";

/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        background: '#F5F5F7',
        surface: '#FFFFFF',
        'surface-hover': '#F0F0F2',
        'text-primary': '#1D1D1F',
        'text-secondary': '#86868B',
        'text-tertiary': '#C7C7CC',
        'accent-gold': '#D4AF37',
        'accent-gold-light': '#F5E6A3',
        'border-light': '#E5E5EA',
        'border-medium': '#D2D2D7',
        danger: '#FF3B30',
        success: '#34C759',
        warning: '#FF9500',
        info: '#007AFF',
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive) / <alpha-value>)",
          foreground: "hsl(var(--destructive-foreground) / <alpha-value>)",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      fontFamily: {
        display: ['Inter', 'SF Pro Display', '-apple-system', 'sans-serif'],
        body: ['Inter', 'PingFang SC', 'Noto Sans SC', 'sans-serif'],
      },
      borderRadius: {
        xl: "calc(var(--radius) + 4px)",
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
        xs: "calc(var(--radius) - 6px)",
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
        "pulse-gold": {
          "0%, 100%": { boxShadow: "inset 0 -2px 0 rgba(212, 175, 55, 0.6)" },
          "50%": { boxShadow: "inset 0 -2px 0 rgba(212, 175, 55, 0.2)" },
        },
        "pulse-red": {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.4" },
        },
        "flash-finished": {
          "0%, 100%": { opacity: "0.5" },
          "50%": { opacity: "0.2" },
        },
        "progress-fill": {
          from: { width: "0%" },
          to: { width: "100%" },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        "pulse-gold": "pulse-gold 1s ease-in-out infinite",
        "pulse-red": "pulse-red 0.5s ease-in-out infinite",
        "flash-finished": "flash-finished 0.6s ease-in-out infinite",
      },
    },
  },
  plugins: [tailwindcssAnimate],
};
