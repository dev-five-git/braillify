namespace Braillify.Tests;

using Xunit;

public sealed class BraillifyTests
{
    [Fact]
    public void EncodeToUnicode_SimpleKorean_ReturnsExpectedBraille()
    {
        // Arrange
        const string input = "안녕하세요";
        const string expected = "⠣⠒⠉⠻⠚⠠⠝⠬";

        // Act
        var result = Braillify.EncodeToUnicode(input);

        // Assert
        Assert.Equal(expected, result);
    }

    [Fact]
    public void Encode_SimpleKorean_ReturnsNonEmptyByteArray()
    {
        // Arrange
        const string input = "안녕";

        // Act
        var result = Braillify.Encode(input);

        // Assert
        Assert.NotEmpty(result);
    }

    [Fact]
    public void EncodeToBrailleFont_SimpleKorean_ReturnsNonEmptyString()
    {
        // Arrange
        const string input = "테스트";

        // Act
        var result = Braillify.EncodeToBrailleFont(input);

        // Assert
        Assert.NotEmpty(result);
    }

    [Fact]
    public void EncodeToUnicode_NullInput_ThrowsArgumentNullException()
    {
        // Act & Assert
        Assert.Throws<ArgumentNullException>(() => Braillify.EncodeToUnicode(null!));
    }

    [Theory]
    [InlineData("상상이상의", "⠇⠶⠇⠶⠕⠇⠶⠺")]
    [InlineData("1,000", "⠼⠁⠂⠚⠚⠚")]
    [InlineData("ATM", "⠠⠠⠁⠞⠍")]
    public void EncodeToUnicode_VariousInputs_ReturnsExpectedResults(string input, string expected)
    {
        // Act
        var result = Braillify.EncodeToUnicode(input);

        // Assert
        Assert.Equal(expected, result);
    }

    [Fact]
    public void EncodeToUnicode_EmptyString_ReturnsEmptyString()
    {
        // Arrange
        const string input = "";

        // Act
        var result = Braillify.EncodeToUnicode(input);

        // Assert
        Assert.Equal(string.Empty, result);
    }

    [Fact]
    public void EncodeInContext_Science_ReadsTheUnitFormula()
    {
        Assert.Equal("⠴⠏⠠⠕⠠⠓", Braillify.EncodeToUnicode("pOH", "science"));
        Assert.Equal("⠴⠏⠠⠠⠕⠓⠲", Braillify.EncodeToUnicode("pOH", "korean"));
        Assert.Equal("⠴⠏⠠⠕⠠⠓", Braillify.EncodeToBrailleFont("pOH", "science"));
        Assert.Equal(6, Braillify.Encode("pOH", "science").Length);
    }

    [Fact]
    public void EncodeInContext_UnknownContext_ThrowsBraillifyException()
    {
        Assert.Throws<BraillifyException>(() => Braillify.EncodeToUnicode("pOH", "chemistry"));
        Assert.Throws<BraillifyException>(() => Braillify.Encode("pOH", "chemistry"));
    }

    [Fact]
    public void EncodeInContext_NullContext_ThrowsArgumentNullException()
    {
        Assert.Throws<ArgumentNullException>(() => Braillify.EncodeToUnicode("pOH", null!));
        Assert.Throws<ArgumentNullException>(() => Braillify.Encode("pOH", null!));
    }

    [Fact]
    public void Encode_EmptyString_ReturnsEmptyArray()
    {
        // Arrange
        const string input = "";

        // Act
        var result = Braillify.Encode(input);

        // Assert
        Assert.Empty(result);
    }
}
