using namespace std;

class BaseControlBlock {
   public:
    virtual void Decrement() = 0;
    virtual void Increment() = 0;
    virtual int GetCount() = 0;
    virtual ~BaseControlBlock() = default;
};

class DerivedControlBlock : public BaseControlBlock {
   public:
    DerivedControlBlock() : count(1) {}
    void Decrement() override {
        count--;
    }
    void Increment() override {
        count++;
    }
    int GetCount() override {
        return count;
    }
    ~DerivedControlBlock() = default;
}
