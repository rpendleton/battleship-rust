# Battleship xcframework Build Makefile

# Project configuration
PROJECT_NAME = battleship
XCFRAMEWORK_NAME = Battleship
LIB_NAME = lib$(PROJECT_NAME).a
HEADERS_DIR = xcframework/headers
TARGET_DIR = target
XCFRAMEWORK_OUTPUT = $(TARGET_DIR)/$(XCFRAMEWORK_NAME).xcframework
IPHONEOS_DEPLOYMENT_TARGET = 18.0

# Rust targets for iOS
IOS_SIM_TARGET = aarch64-apple-ios-sim
IOS_DEVICE_TARGET = aarch64-apple-ios
IOS_SIM_LIB = $(TARGET_DIR)/$(IOS_SIM_TARGET)/release/$(LIB_NAME)
IOS_DEVICE_LIB = $(TARGET_DIR)/$(IOS_DEVICE_TARGET)/release/$(LIB_NAME)

# Build configuration
CARGO_FLAGS = --release

# Colors for output
RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[0;33m
BLUE = \033[0;34m
NC = \033[0m # No Color

.PHONY: all xcframework ios-sim ios-device install-targets clean test help

# Default target
all: xcframework

# Help target
help:
	@echo "$(BLUE)Battleship Build System$(NC)"
	@echo ""
	@echo "Available targets:"
	@echo "  $(GREEN)all$(NC)              - Build complete xcframework (default)"
	@echo "  $(GREEN)xcframework$(NC)      - Build xcframework for iOS simulator and device"
	@echo "  $(GREEN)ios-sim$(NC)          - Build for iOS simulator only"
	@echo "  $(GREEN)ios-device$(NC)       - Build for iOS device only"
	@echo "  $(GREEN)clean$(NC)            - Clean build artifacts"
	@echo "  $(GREEN)test$(NC)             - Run tests in release mode"
	@echo "  $(GREEN)install-targets$(NC)  - Install required Rust targets"
	@echo "  $(GREEN)help$(NC)             - Show this help message"

ios-sim:
	@echo "$(BLUE)Building for iOS Simulator ($(IOS_SIM_TARGET))...$(NC)"
	IPHONEOS_DEPLOYMENT_TARGET=$(IPHONEOS_DEPLOYMENT_TARGET) \
	  cargo build $(CARGO_FLAGS) --target $(IOS_SIM_TARGET)
	@echo "$(GREEN)✓ iOS Simulator build complete$(NC)"

ios-device:
	@echo "$(BLUE)Building for iOS Device ($(IOS_DEVICE_TARGET))...$(NC)"
	IPHONEOS_DEPLOYMENT_TARGET=$(IPHONEOS_DEPLOYMENT_TARGET) \
	  cargo build $(CARGO_FLAGS) --target $(IOS_DEVICE_TARGET)
	@echo "$(GREEN)✓ iOS Device build complete$(NC)"

xcframework: ios-sim ios-device
	@echo "$(BLUE)Creating universal xcframework...$(NC)"
	@if [ -d "$(XCFRAMEWORK_OUTPUT)" ]; then \
		echo "$(YELLOW)Removing existing xcframework...$(NC)"; \
		rm -rf "$(XCFRAMEWORK_OUTPUT)"; \
	fi

	xcodebuild -create-xcframework \
		-library "$(IOS_SIM_LIB)" \
		-headers "$(HEADERS_DIR)" \
		-library "$(IOS_DEVICE_LIB)" \
		-headers "$(HEADERS_DIR)" \
		-output "$(XCFRAMEWORK_OUTPUT)"

	@echo "$(GREEN)✓ Universal xcframework created at $(XCFRAMEWORK_OUTPUT)$(NC)"

test:
	@echo "$(BLUE)Running tests in release mode...$(NC)"
	cargo test --release -- --nocapture
	@echo "$(GREEN)✓ Tests completed$(NC)"

clean:
	@echo "$(YELLOW)Cleaning all build artifacts...$(NC)"
	cargo clean
	rm -rf "$(XCFRAMEWORK_OUTPUT)"
	@echo "$(GREEN)✓ All artifacts cleaned$(NC)"

# Development workflow
dev: clean xcframework verify-xcframework
	@echo "$(GREEN)✓ Development build complete$(NC)"

# Check prerequisites
check-prereqs:
	@echo "$(BLUE)Checking prerequisites...$(NC)"
	@command -v rustc >/dev/null 2>&1 || { echo "$(RED)❌ Rust not installed$(NC)"; exit 1; }
	@command -v cargo >/dev/null 2>&1 || { echo "$(RED)❌ Cargo not installed$(NC)"; exit 1; }
	@command -v xcodebuild >/dev/null 2>&1 || { echo "$(RED)❌ Xcode command line tools not installed$(NC)"; exit 1; }
	@echo "$(GREEN)✓ All prerequisites satisfied$(NC)"

# Show build info
info:
	@echo "$(BLUE)Build Configuration:$(NC)"
	@echo "  Project: $(PROJECT_NAME)"
	@echo "  xcframework: $(XCFRAMEWORK_NAME)"
	@echo "  Library: $(LIB_NAME)"
	@echo "  Headers: $(HEADERS_DIR)"
	@echo "  Output: $(XCFRAMEWORK_OUTPUT)"
	@echo "  iOS Simulator Target: $(IOS_SIM_TARGET)"
	@echo "  iOS Device Target: $(IOS_DEVICE_TARGET)"
	@echo ""
	@echo "$(BLUE)Current Status:$(NC)"
	@if [ -f "$(IOS_SIM_LIB)" ]; then \
		echo "  $(GREEN)✓$(NC) iOS Simulator library exists"; \
	else \
		echo "  $(RED)❌$(NC) iOS Simulator library missing"; \
	fi
	@if [ -f "$(IOS_DEVICE_LIB)" ]; then \
		echo "  $(GREEN)✓$(NC) iOS Device library exists"; \
	else \
		echo "  $(RED)❌$(NC) iOS Device library missing"; \
	fi
	@if [ -d "$(XCFRAMEWORK_OUTPUT)" ]; then \
		echo "  $(GREEN)✓$(NC) xcframework exists"; \
	else \
		echo "  $(RED)❌$(NC) xcframework missing"; \
	fi

install-targets:
	@echo "$(BLUE)Installing Rust targets...$(NC)"
	rustup target add $(IOS_SIM_TARGET)
	rustup target add $(IOS_DEVICE_TARGET)
	@echo "$(GREEN)✓ Rust targets installed$(NC)"
